use crate::{util::OnDropDefer, WindowMessenger, WindowInstance, AppMessage, ffi::ResponseBinding};
use ash::{extensions::{ext, khr}, vk::{self, Handle}};
use std::{ffi::CStr, mem::MaybeUninit};

#[derive(thiserror::Error, Debug)]
pub enum RendererError {
    #[error("Failed to load Video Driver: {0}")]
    FailedToLoadDriver(#[from] ash::LoadingError),
    #[error("Failed to create Vulkan instance, Vulkan error code: {0}")]
    FailedToCreateInstance(vk::Result),
    #[error("Failed to create Vulkan debug messenger, Vulkan error code: {0}")]
    FailedToCreateDebugMessenger(vk::Result),
    #[error("Failed to query info from the Vulkan driver. Vulkan error code: {0}")]
    VulkanInfoQueryFailed(vk::Result),
    /// This error contains a list of the validation layers that were requested
    /// and were available. It indicates however, that some validation layers
    /// were unavailable, so the returned list only contains the available
    /// subset of those.
    #[error("The program requires some validation layers that are not available")]
    UnavailableValidationLayers(Box<[*const i8]>),
    #[error("Failed to find an video adapter (GPU) supporting Vulkan")]
    NoAvailableVideoAdapter,
    #[error("Failed to create Vulkan device, Vulkan error code: {0}")]
    FailedToCreateDevice(vk::Result),
}

mod queue;

pub struct Renderer {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    device: ash::Device,
    graphics_queue: vk::Queue,
    surface: vk::SurfaceKHR,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        log::debug!("Destroying Vulkan surface");
        let surface_ext = khr::Surface::new(&self.entry, &self.instance);
        unsafe { surface_ext.destroy_surface(self.surface, None) };

        log::debug!("Destroying Vulkan device");
        unsafe { self.device.destroy_device(None) };

        if let Some(debug_messenger) = self.debug_messenger {
            log::debug!("Destroying Vulkan debug utils");
            let debug_utils = ext::DebugUtils::new(&self.entry, &self.instance);
            unsafe { debug_utils.destroy_debug_utils_messenger(debug_messenger, None) };
        }

        log::debug!("Destroying Vulkan instance");
        unsafe { self.instance.destroy_instance(None) };
    }
}
impl Renderer {
    pub fn new(window: WindowInstance, messenger: WindowMessenger) -> Result<Renderer, RendererError> {
        // And it begins!
        let entry = unsafe { ash::Entry::load()? };

        let app_info = vk::ApplicationInfo {
            p_application_name: b"Faisca App\0" as *const u8 as *const i8,
            application_version: vk::make_api_version(0, 1, 0, 0),
            p_engine_name: b"Faisca\0" as *const u8 as *const i8,
            engine_version: vk::make_api_version(0, 1, 0, 0),
            // Using API version 1.0, we will support most devices.
            api_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };

        // Instance extensions are driver extensions that are useful independently
        // of any specific device
        let instance_extensions_array = Renderer::get_instance_extensions(&entry)?;
        // Validation layers are like extensions, but used to make debugging
        // simpler, as well as giving warnings in case we do something outside
        // of what is allowed by the Vulkan specification.
        let validation_layers_array = Renderer::get_validation_layers(&entry)?;

        let debug_messenger_info = vk::DebugUtilsMessengerCreateInfoEXT {
            // We will accept messages of these severities...
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            // And of these kinds
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING,
            // Here is where we set our callback function
            pfn_user_callback: Some(_vk_debug_callback),
            // We are explicit about the fact the debugger carries no special
            // data. The logging system works on a process-wide (global) level.
            p_user_data: std::ptr::null_mut(),
            ..Default::default()
        };

        let instance_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            // This application cannot work without at least a surface extension
            // (because we need to display to a window). This means we don't check
            // if `instance_extensions_array` is empty or not, as the application
            // would've ran into an error erlier if that were the case. The function
            // `get_instance_extensions` called earlier makes the check.
            pp_enabled_extension_names: instance_extensions_array.as_ptr(),
            enabled_extension_count: instance_extensions_array.len().try_into().unwrap(),
            // We enable layers only if there are layers to be enabled
            pp_enabled_layer_names: if validation_layers_array.len() > 0 {
                validation_layers_array.as_ptr()
            } else {
                std::ptr::null()
            },
            enabled_layer_count: validation_layers_array.len().try_into().unwrap(),
            // In debug mode, we set an extra header so that we can debug the
            // creation and destruction of the instance.
            p_next: if crate::DEBUG_ENABLED {
                &debug_messenger_info as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const std::ffi::c_void
            } else {
                std::ptr::null()
            },
            ..Default::default()
        };

        log::debug!("VkInstanceCreateInfo: {instance_info:#?}");

        let instance = unsafe { entry.create_instance(&instance_info, None) }
            .map_err(RendererError::FailedToCreateInstance)?;

        // I'll use this ~beautiful~ `OnDropDefer` thingie to make sure we
        // destroy the things we create even in case we return earlier or panic.
        let instance = OnDropDefer::new(instance, |i| {
            unsafe { i.destroy_instance(None) };
        });

        log::debug!("Vulkan Instance created");

        // In debug mode, we make a debug messenger, that will give us feedback,
        // specially about all that validation thing we talked about earlier.
        // This is how the driver tells us a bit about what it is doing on its
        // end, and gives feedback about how badly we're working on our end.
        let debug_messenger = if crate::DEBUG_ENABLED {
            let debug_utils = ext::DebugUtils::new(&entry, instance.as_ref());
            let messenger =
                unsafe { debug_utils.create_debug_utils_messenger(&debug_messenger_info, None) }
                    .map_err(RendererError::FailedToCreateDebugMessenger)?;

            let messenger = OnDropDefer::new(messenger, |m| {
                let debug_utils = ext::DebugUtils::new(&entry, instance.as_ref());
                unsafe { debug_utils.destroy_debug_utils_messenger(m, None) }
            });

            Some(messenger)
        } else {
            None
        };

        let selected_physical_device = Self::select_physical_device(instance.as_ref())?;

        let device = Self::create_device(&entry, instance.as_ref(), selected_physical_device)?;
        let device = OnDropDefer::new(device, |d| unsafe {
            d.destroy_device(None);
        });

        let queue_indices =
            queue::QueueFamilyIndices::fetch(instance.as_ref(), selected_physical_device);
        let graphics_queue =
            unsafe { device.as_ref().get_device_queue(queue_indices.graphics_family.unwrap(), 0) };

        let mut surface: MaybeUninit<u64> = MaybeUninit::uninit();
        let mut binding = unsafe {
            ResponseBinding::new(surface.as_mut_ptr() as *mut std::ffi::c_void)
        };
        messenger.send(window, &AppMessage::CreateVulkanSurface {
            instance: instance.as_ref().handle().as_raw(),
            out_binding: &mut binding as *mut ResponseBinding,
        });
        binding.wait();

        let surface = vk::SurfaceKHR::from_raw(unsafe { surface.assume_init() });

        let device = device.take();
        let debug_messenger = debug_messenger.map(OnDropDefer::take);
        let instance = instance.take();

        Ok(Renderer {
            entry,
            instance,
            debug_messenger,
            device,
            graphics_queue,
            surface,
        })
    }

    /// This function checks whether or not the driver supports the instance
    /// extensions we required, returning an error otherwise. In case the driver
    /// supports the extensions, we return the extension list so that it can be
    /// used to initialize the driver.
    ///
    /// This function also returns an error in case it fails to query the driver
    /// about supported extensions.
    fn get_instance_extensions(entry: &ash::Entry) -> Result<Box<[*const i8]>, RendererError> {
        let properties = entry
            .enumerate_instance_extension_properties(None)
            .map_err(RendererError::VulkanInfoQueryFailed)?;
        if crate::DEBUG_ENABLED {
            let mut ext_names = String::new();
            for prop in &properties {
                let prop_name =
                    unsafe { CStr::from_ptr(prop.extension_name.as_ptr()) }.to_string_lossy();
                ext_names.push_str(&format!("\n{prop_name}"));
            }
            log::debug!("Available Vulkan extensions:{ext_names}");
        }

        let mut required_ext: Vec<*const i8> = crate::VK_INSTANCE_EXTENSIONS_VEC
            .read()
            .unwrap()
            .iter()
            .map(|&usize_ptr_rep| usize_ptr_rep as *const i8)
            .collect();

        if crate::DEBUG_ENABLED {
            required_ext.push(crate::VK_EXT_DEBUG_UTILS_EXTENSION_NAME.as_ptr() as *const i8);

            let mut ext_names = String::new();
            for ext in required_ext.iter().cloned() {
                let ext_name = unsafe { CStr::from_ptr(ext) }.to_string_lossy();
                ext_names.push_str(&format!("\n{ext_name}"));
            }
            log::debug!("Required extensions:{ext_names}");
        }

        Ok(required_ext.into_boxed_slice())
    }

    /// This function fetches the list of validation layers that we want to use.
    /// It also checks whether the validation layers we want are available. If
    /// we fail to query the driver for supported extensions, the driver will
    /// return the error `VulkanInfoQueryFailed`. If only a subset of the layers
    /// are available (including in case none are available), it will return
    /// `UnavailableValidationLayers`, containing the subset of the requested
    /// layers that was available.
    fn get_validation_layers(entry: &ash::Entry) -> Result<Box<[*const i8]>, RendererError> {
        let all_available_layers = entry
            .enumerate_instance_layer_properties()
            .map_err(RendererError::VulkanInfoQueryFailed)?;
        if crate::DEBUG_ENABLED {
            let mut ext_names = String::new();
            for layer in &all_available_layers {
                let ext_name =
                    unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) }.to_string_lossy();
                ext_names.push_str(&format!("\n{ext_name}"));
            }
            log::debug!("Available validation layers:{ext_names}");
        }

        let requested_and_available: Vec<*const i8> = crate::VK_VALIDATION_LAYERS
            .iter()
            .cloned()
            .map(|cstr_ptr| unsafe { CStr::from_ptr(cstr_ptr) })
            .filter(|&requested| {
                all_available_layers
                    .iter()
                    .map(|layer| unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) })
                    .find(|&available| requested == available)
                    .is_some()
            })
            .map(CStr::as_ptr)
            .collect();

        if crate::DEBUG_ENABLED {
            // Check if all of the requested layers is available
            if requested_and_available.len() == crate::VK_VALIDATION_LAYERS.len() {
                Ok(Box::new(crate::VK_VALIDATION_LAYERS))
            } else {
                // If not, then we give an error and pass the list of the ones
                // available
                log::error!("Not all requested validation layers were available.");
                Err(RendererError::UnavailableValidationLayers(
                    requested_and_available.into_boxed_slice(),
                ))
            }
        } else {
            Ok(Box::new([]))
        }
    }

    /// Picks the first device it finds that supports the operations we want to
    /// do (the first suitable device).
    fn select_physical_device(
        instance: &ash::Instance,
    ) -> Result<vk::PhysicalDevice, RendererError> {
        let devices = unsafe { instance.enumerate_physical_devices() }
            .map_err(RendererError::VulkanInfoQueryFailed)?;

        if devices.len() == 0 {
            Err(RendererError::NoAvailableVideoAdapter)
        } else {
            devices
                .iter()
                .cloned()
                .find(|&d| Self::check_physical_device_suitability(instance, d))
                .ok_or(RendererError::NoAvailableVideoAdapter)
        }
    }

    /// Checks if a given physical device is capable of performing the
    /// operations the application needs.
    fn check_physical_device_suitability(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let indices = queue::QueueFamilyIndices::fetch(instance, physical_device);
        indices.has_all()
    }

    fn create_device(
        entry: &ash::Entry,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<ash::Device, RendererError> {
        let indices = queue::QueueFamilyIndices::fetch(instance, physical_device);

        let priority = 1.0_f32;
        let queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index: indices.graphics_family.unwrap(),
            queue_count: 1,
            p_queue_priorities: &priority as *const f32,
            ..Default::default()
        };

        let device_features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        let validation_layers = if crate::DEBUG_ENABLED {
            Self::get_validation_layers(entry)?
        } else {
            Box::new([])
        };

        let device_info = vk::DeviceCreateInfo {
            p_queue_create_infos: &queue_create_info as *const vk::DeviceQueueCreateInfo,
            queue_create_info_count: 1,
            p_enabled_features: &device_features as *const vk::PhysicalDeviceFeatures,
            enabled_extension_count: 0,
            pp_enabled_layer_names: if validation_layers.len() > 0 {
                validation_layers.as_ptr()
            } else {
                std::ptr::null()
            },
            enabled_layer_count: validation_layers.len().try_into().unwrap(),
            ..Default::default()
        };

        log::debug!("VkDeviceCreateInfo: {device_info:#?}");
        log::debug!("VkPhysicalDeviceFeatures: {device_features:#?}");

        unsafe { instance.create_device(physical_device, &device_info, None) }
            .map_err(RendererError::FailedToCreateDevice)
    }
}

unsafe extern "system" fn _vk_debug_callback(
    msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    use vk::DebugUtilsMessageSeverityFlagsEXT as VkMSev;
    use vk::DebugUtilsMessageTypeFlagsEXT as VkMType;

    let log_level = if msg_severity.contains(VkMSev::ERROR) {
        log::Level::Error
    } else if msg_severity.contains(VkMSev::WARNING) {
        log::Level::Warn
    } else if msg_type.contains(VkMType::PERFORMANCE) {
        log::Level::Info
    } else if msg_type.contains(VkMType::VALIDATION) {
        log::Level::Debug
    } else {
        log::Level::Trace
    };

    log::log!(
        log_level,
        "[VKMSG {msg_type:?}] {}",
        unsafe { CStr::from_ptr((*callback_data).p_message) }.to_string_lossy()
    );

    vk::FALSE
}
