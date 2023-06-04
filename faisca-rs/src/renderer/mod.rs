use crate::{ffi::ResponseBinding, util::OnDropDefer, AppMessage, WindowInstance, WindowMessenger};
use ash::{
    extensions::{ext, khr},
    vk::{self, Handle, MemoryPropertyFlags},
};
use std::{ffi::CStr, mem::MaybeUninit};

use self::{buffer::VirtualBuffer, resources::RendererResourceKeeper};

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

    #[error("Failed to create Vulkan surface")]
    FailedToCreateVulkanSurface,
    #[error("Failed to find a video adapter (GPU) supporting Vulkan")]
    NoAvailableVideoAdapter,
    #[error("Failed to find a video adapter (GPU) that this application supports")]
    NoSupportedVideoAdapter,
    #[error("Failed to create Vulkan device, Vulkan error code: {0}")]
    FailedToCreateDevice(vk::Result),
    #[error("Failed to create Vulkan swapchain, Vulkan error code: {0}")]
    FailedToCreateSwapchain(vk::Result),
    #[error("Failed to create Vulkan image view, Vulkan error code: {0}")]
    FailedToCreateImageView(vk::Result),
    #[error("Failed to create Vulkan shader module, Vulkan error code: {0}")]
    FailedToCreateShaderModule(vk::Result),
    #[error("Failed to create Vulkan pipeline layout, Vulkan error code: {0}")]
    FailedToCreatePipelineLayout(vk::Result),
    #[error("Failed to create Vulkan render pass, Vulkan error code: {0}")]
    FailedToCreateRenderPass(vk::Result),
    #[error("Failed to create Vulkan graphics pipeline, Vulkan error code: {0}")]
    FailedToCreateGraphicsPipeline(vk::Result),
    #[error("Failed to create Vulkan framebuffer, Vulkan error code: {0}")]
    FailedToCreateFramebuffer(vk::Result),
    #[error("Failed to create Vulkan command pool, Vulkan error code: {0}")]
    FailedToCreateCommandPool(vk::Result),
    #[error("Failed to create Vulkan command buffer, Vulkan error code: {0}")]
    FailedToCreateCommandBuffer(vk::Result),
    #[error("An error occurred while recording commands, Vulkan error code: {0}")]
    CommandBufferRecordingError(vk::Result),
    #[error("Failed to create Vulkan sync object, Vulkan error code: {0}")]
    FailedToCreateSyncObject(vk::Result),
    #[error("Failed to create Vulkan buffer, Vulkan error code: {0}")]
    FailedToCreateBuffer(vk::Result),
    #[error(
        "Failed to find suitable memory type: {memory_type_flags:x}, {memory_property_flags:?}"
    )]
    UnavailableMemoryType {
        memory_type_flags: u32,
        memory_property_flags: MemoryPropertyFlags,
    },
    #[error("Failed to allocate memory, Vulkan error code: {0}")]
    MemAllocError(vk::Result),
    #[error("Failed to map memory to buffer, Vulkan error code: {0}")]
    FailedToMapBufferMemory(vk::Result),
    #[error("The object could not be accepted or created because it was too big")]
    ObjectTooBig,

    #[error("Failed to draw Vulkan frame, Vulkan error code: {0}")]
    FailedToDrawFrame(vk::Result),
}

mod buffer;
mod queue;
mod resources;
mod swapchain_info;
mod vertex;

const MAX_CONCURRENT_FRAMES: usize = 2;

use vertex::{Point2DColorRGBVertex, Vector2, Vector3};

#[rustfmt::skip]
static VERTICES: [Point2DColorRGBVertex; 4] = [
    Point2DColorRGBVertex { point: Vector2([ 0.5, -0.5]), color: Vector3([ 1.0,  0.0,  0.0]) },
    Point2DColorRGBVertex { point: Vector2([ 0.5,  0.5]), color: Vector3([ 0.0,  1.0,  0.0]) },
    Point2DColorRGBVertex { point: Vector2([-0.5,  0.5]), color: Vector3([ 0.0,  0.0,  1.0]) },
    Point2DColorRGBVertex { point: Vector2([-0.5, -0.5]), color: Vector3([ 1.0,  0.0,  0.0]) },
];

pub struct Renderer {
    vk_res: RendererResourceKeeper,
    #[allow(unused)]
    entry: ash::Entry,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain_img_extent: vk::Extent2D,
    command_buffers: Vec<vk::CommandBuffer>,
    current_frame: usize,

    test_vertex_buffer: VirtualBuffer,
    test_index_buffer: VirtualBuffer,
}

impl Renderer {
    pub fn new(
        window: WindowInstance,
        messenger: &WindowMessenger,
    ) -> Result<Renderer, RendererError> {
        // And it begins!
        let entry = unsafe { ash::Entry::load()? };

        let mut vk_res = RendererResourceKeeper::new();

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

        unsafe {
            *vk_res.instance_mut() = Some(
                entry
                    .create_instance(&instance_info, None)
                    .map_err(RendererError::FailedToCreateInstance)?,
            )
        };

        log::debug!("Vulkan Instance created");

        unsafe {
            *vk_res.surface_loader_mut() = Some(khr::Surface::new(&entry, vk_res.instance()));
            *vk_res.debug_loader_mut() = Some(ext::DebugUtils::new(&entry, vk_res.instance()));
        };

        // In debug mode, we make a debug messenger, that will give us feedback,
        // specially about all that validation thing we talked about earlier.
        // This is how the driver tells us a bit about what it is doing on its
        // end, and gives feedback about how badly we're working on our end.
        let debug_messenger = if crate::DEBUG_ENABLED {
            let messenger = unsafe {
                vk_res
                    .debug_loader()
                    .create_debug_utils_messenger(&debug_messenger_info, None)
            }
            .map_err(RendererError::FailedToCreateDebugMessenger)?;

            messenger
        } else {
            vk::DebugUtilsMessengerEXT::null()
        };

        unsafe { *vk_res.debug_messenger_mut() = debug_messenger };

        // Here we basically ask the Window for a SurfaceKHR handle. The window code
        // will do something platform-specific in order to acquire this handle for us.
        let mut surface: MaybeUninit<u64> = MaybeUninit::uninit();
        let binding =
            unsafe { ResponseBinding::new(surface.as_mut_ptr() as *mut std::ffi::c_void) };
        messenger.send(
            window,
            &AppMessage::CreateVulkanSurface {
                instance: vk_res.instance().handle().as_raw(),
                out_binding: &binding as *const ResponseBinding,
            },
        );
        binding.wait();

        unsafe {
            *vk_res.surface_mut() = vk::SurfaceKHR::from_raw(surface.assume_init());
        }
        // It is possible that the handle creation fails, in which case we receive a
        // null handle.
        if vk_res.surface() == vk::SurfaceKHR::null() {
            return Err(RendererError::FailedToCreateVulkanSurface);
        }

        let selected_physical_device = Self::select_physical_device(
            vk_res.instance(),
            vk_res.surface_loader(),
            vk_res.surface(),
        )?;

        unsafe {
            *vk_res.physical_device_mut() = selected_physical_device;
        }

        // With this, for some physical device, we store the queue indices for the
        // queue falimies we want to use (graphics and present). They may have the
        // same index (be the same family).
        let queue_indices = queue::QueueFamilyIndices::fetch(
            vk_res.instance(),
            vk_res.surface_loader(),
            vk_res.surface(),
            selected_physical_device,
        );

        let device = Self::create_device(
            &entry,
            vk_res.instance(),
            &queue_indices,
            selected_physical_device,
        )?;
        unsafe { *vk_res.device_mut() = Some(device) };

        let graphics_queue = unsafe {
            vk_res
                .device()
                .get_device_queue(queue_indices.graphics_family.unwrap(), 0)
        };

        let present_queue = unsafe {
            vk_res
                .device()
                .get_device_queue(queue_indices.present_family.unwrap(), 0)
        };

        unsafe { *vk_res.queue_families_mut() = queue_indices };

        let swapchain_info = swapchain_info::SwapchainSupportInfo::fetch(
            vk_res.surface_loader(),
            vk_res.surface(),
            selected_physical_device,
        )
        .unwrap();

        unsafe {
            *vk_res.swapchain_loader_mut() =
                Some(khr::Swapchain::new(vk_res.instance(), vk_res.device()));
        }

        let mut extent = MaybeUninit::<vk::Extent2D>::uninit();
        let out_binding =
            unsafe { ResponseBinding::new(extent.as_mut_ptr() as *mut std::ffi::c_void) };
        messenger.send(
            window,
            &AppMessage::QueryViewportExtents {
                out_binding: &out_binding as *const ResponseBinding,
            },
        );
        out_binding.wait();

        let swapchain_img_extent = swapchain_info.select_extent(unsafe { extent.assume_init() });
        let swapchain_img_format = swapchain_info.select_format().unwrap();

        unsafe {
            *vk_res.render_pass_mut() =
                Self::create_render_pass(vk_res.device(), swapchain_img_format.format)?;
        }

        vk_res.create_swapchain(&swapchain_info, swapchain_img_extent)?;

        unsafe {
            let (pipeline, pipeline_layout) = Self::create_graphics_pipeline(
                vk_res.device(),
                swapchain_img_extent,
                vk_res.render_pass(),
            )?;
            *vk_res.pipeline_mut() = pipeline_layout;
            *vk_res.pipeline_layout_mut() = pipeline;
        }

        let command_pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: queue_indices.graphics_family.unwrap(),
            ..Default::default()
        };
        unsafe {
            *vk_res.command_pool_mut() = vk_res
                .device()
                .create_command_pool(&command_pool_info, None)
                .map_err(RendererError::FailedToCreateCommandPool)?;
        }

        let command_buffer_info = vk::CommandBufferAllocateInfo {
            command_pool: vk_res.command_pool(),
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAX_CONCURRENT_FRAMES.try_into().unwrap(),
            ..Default::default()
        };
        let command_buffers = unsafe {
            vk_res
                .device()
                .allocate_command_buffers(&command_buffer_info)
        }
        .map_err(RendererError::FailedToCreateCommandBuffer)?;

        unsafe { vk_res.create_sync_objects(MAX_CONCURRENT_FRAMES)? };

        let vertex_data_len = VERTICES
            .len()
            .checked_mul(std::mem::size_of::<Point2DColorRGBVertex>())
            .unwrap();

        let test_vertex_buffer = unsafe {
            let vertex_data =
                std::slice::from_raw_parts(VERTICES.as_ptr() as *const u8, vertex_data_len);
            vk_res.create_vertex_vbuffer(vertex_data)?
        };
        let test_index_buffer = unsafe {
            let indices = [0, 1, 2, 2, 3, 0u16];
            vk_res.create_index_vbuffer(&indices)?
        };

        Ok(Renderer {
            entry,
            vk_res,
            graphics_queue,
            present_queue,
            swapchain_img_extent,
            command_buffers,
            current_frame: 0,

            test_vertex_buffer,
            test_index_buffer,
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
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> Result<vk::PhysicalDevice, RendererError> {
        let devices = unsafe { instance.enumerate_physical_devices() }
            .map_err(RendererError::VulkanInfoQueryFailed)?;

        if devices.len() == 0 {
            Err(RendererError::NoAvailableVideoAdapter)
        } else {
            devices
                .iter()
                .cloned()
                .find(|&d| {
                    let family_indices =
                        queue::QueueFamilyIndices::fetch(instance, surface_loader, surface, d);

                    swapchain_info::SwapchainSupportInfo::fetch(surface_loader, surface, d)
                        .map(|swapchain_info| {
                            Self::check_physical_device_suitability(
                                instance,
                                &family_indices,
                                &swapchain_info,
                                d,
                            )
                        })
                        .unwrap_or_else(|e| {
                            log::error!("Failed to query device for swapchain support: {e}");
                            false
                        })
                })
                .ok_or(RendererError::NoSupportedVideoAdapter)
        }
    }

    /// Checks if a given physical device is capable of performing the
    /// operations the application needs.
    fn check_physical_device_suitability(
        instance: &ash::Instance,
        family_indices: &queue::QueueFamilyIndices,
        swapchain_info: &swapchain_info::SwapchainSupportInfo,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        family_indices.has_all()
            && Self::check_device_supports_extensions(instance, physical_device)
            && Self::check_swapchain_is_adequate(swapchain_info)
    }

    fn check_device_supports_extensions(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let supported_device_extensions =
            unsafe { instance.enumerate_device_extension_properties(physical_device) }
                .unwrap_or(Vec::new());
        if crate::DEBUG_ENABLED {
            let mut extension_list = String::new();
            for supported_ext in supported_device_extensions.iter() {
                let supported_ext_name =
                    unsafe { CStr::from_ptr(supported_ext.extension_name.as_ptr()) };
                extension_list.push_str(&format!("\n{}", supported_ext_name.to_string_lossy(),));
            }
            log::debug!(
                "Supported device extensions for device {physical_device:?}:{extension_list}"
            );
        }
        crate::VK_REQUIRED_DEVICE_EXTENSIONS
            .iter()
            .map(|&cstr_ptr| unsafe { CStr::from_ptr(cstr_ptr) })
            .all(|required_ext| {
                supported_device_extensions
                    .iter()
                    .map(|prop| prop.extension_name.as_ptr())
                    .map(|cstr_ptr| unsafe { CStr::from_ptr(cstr_ptr) })
                    .find(|&supported_ext| required_ext == supported_ext)
                    .is_some()
            })
    }

    fn check_swapchain_is_adequate(swapchain_info: &swapchain_info::SwapchainSupportInfo) -> bool {
        !swapchain_info.formats.is_empty() && !swapchain_info.present_modes.is_empty()
    }

    fn create_device(
        entry: &ash::Entry,
        instance: &ash::Instance,
        family_indices: &queue::QueueFamilyIndices,
        physical_device: vk::PhysicalDevice,
    ) -> Result<ash::Device, RendererError> {
        let mut queue_create_infos = Vec::new();
        // Using a set, if there are any repeated queue indices, they will be
        // reduced to a single one.
        let unique_queue_indices = std::collections::HashSet::from([
            family_indices.graphics_family.unwrap(),
            family_indices.present_family.unwrap(),
        ]);

        let priority = 1.0_f32;
        for unique_queue_index in unique_queue_indices {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                queue_family_index: unique_queue_index,
                queue_count: 1,
                p_queue_priorities: &priority as *const f32,
                ..Default::default()
            };
            queue_create_infos.push(queue_create_info);
        }

        let device_features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        let validation_layers = if crate::DEBUG_ENABLED {
            Self::get_validation_layers(entry)?
        } else {
            Box::new([])
        };

        let device_info = vk::DeviceCreateInfo {
            p_queue_create_infos: queue_create_infos.as_ptr(),
            queue_create_info_count: queue_create_infos.len().try_into().unwrap(),
            p_enabled_features: &device_features as *const vk::PhysicalDeviceFeatures,
            pp_enabled_extension_names: crate::VK_REQUIRED_DEVICE_EXTENSIONS.as_ptr(),
            enabled_extension_count: crate::VK_REQUIRED_DEVICE_EXTENSIONS
                .len()
                .try_into()
                .unwrap(),
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

    fn create_render_pass(
        device: &ash::Device,
        img_format: vk::Format,
    ) -> Result<vk::RenderPass, RendererError> {
        // This structrue defines what we do with the images we receive to
        // render into.
        let color_attachment = vk::AttachmentDescription {
            format: img_format,

            samples: vk::SampleCountFlags::TYPE_1,

            // When we get a new image to render, we clear it
            load_op: vk::AttachmentLoadOp::CLEAR,
            // But we save the things we render into it.
            store_op: vk::AttachmentStoreOp::STORE,

            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,

            // We don't care about the format the image has when we get it
            initial_layout: vk::ImageLayout::UNDEFINED,
            // But we want it to be ready for presenting when we render it
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,

            ..Default::default()
        };

        // These attachments will sort of bind to our shaders if I understand
        // correctly, inside the pipeline. Our fragment shader outputs color, so
        // we choose a COLOR_ATTACHMENT_OPTIMAL layout.
        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            // Our attachment is for color, so we choose it as the optimal
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            // The index of the attachment on this *ARRAY* is equivalent to the
            // index in the `layout(location = X) out vec4 outColor` that the
            // fragment shader uses.
            p_color_attachments: &color_attachment_ref as *const vk::AttachmentReference,
            ..Default::default()
        };

        let render_pass_info = vk::RenderPassCreateInfo {
            attachment_count: 1,
            p_attachments: &color_attachment as *const vk::AttachmentDescription,
            subpass_count: 1,
            p_subpasses: &subpass as *const vk::SubpassDescription,
            dependency_count: 1,
            p_dependencies: &dependency as *const vk::SubpassDependency,
            ..Default::default()
        };

        unsafe { device.create_render_pass(&render_pass_info, None) }
            .map_err(RendererError::FailedToCreateRenderPass)
    }

    fn create_graphics_pipeline(
        device: &ash::Device,
        viewport_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> Result<(vk::PipelineLayout, vk::Pipeline), RendererError> {
        // We put the shaders into u32 vecs to make sure we satisfy alignment
        // requirements.
        let vertex_shader_code = include_bytes!("spir_v/example_vertex_shader.spv")
            .chunks_exact(4)
            .map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<u32>>();
        let fragment_shader_code = include_bytes!("spir_v/example_fragment_shader.spv")
            .chunks_exact(4)
            .map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<u32>>();

        let vertex_shader_module = OnDropDefer::new(
            Self::create_shader_module(device, &vertex_shader_code)?,
            |smodule| {
                log::debug!("Defered shader module destroy called");
                unsafe { device.destroy_shader_module(smodule, None) };
            },
        );
        let fragment_shader_module = OnDropDefer::new(
            Self::create_shader_module(device, &fragment_shader_code)?,
            |smodule| {
                log::debug!("Defered shader module destroy called");
                unsafe { device.destroy_shader_module(smodule, None) };
            },
        );

        const MAIN_ARR: &'static [u8] = b"main\0";
        const MAIN_CSTR: *const i8 = MAIN_ARR as *const [u8] as *const i8;

        let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: *vertex_shader_module.as_ref(),
            p_name: MAIN_CSTR,
            ..Default::default()
        };
        let fragment_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: *fragment_shader_module.as_ref(),
            p_name: MAIN_CSTR,
            ..Default::default()
        };

        let shader_stages = [vertex_shader_stage_info, fragment_shader_stage_info];

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let vertex_layout = Point2DColorRGBVertex::layout();
        let mut attr_descriptions = vec![Default::default(); vertex_layout.num_components()];
        vertex_layout.vulkan_describe_vertex_attributes(0, attr_descriptions.as_mut());
        let binding_description =
            vertex_layout.vulkan_vertex_input_binding_description(0, vk::VertexInputRate::VERTEX);

        let vertex_input_stage_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &binding_description as *const _,
            vertex_attribute_description_count: attr_descriptions.len().try_into().unwrap(),
            p_vertex_attribute_descriptions: attr_descriptions.as_ptr(),
            ..Default::default()
        };

        let input_assembly_stage_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: viewport_extent.width as f32,
            height: viewport_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: viewport_extent,
        };

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states.len().try_into().unwrap(),
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let viewport_state_info = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            p_viewports: &viewport,
            scissor_count: 1,
            p_scissors: &scissor,
            ..Default::default()
        };

        let rasterization_state_info = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,

            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0,

            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,

            depth_bias_enable: vk::FALSE,
            ..Default::default()
        };

        log::debug!("VkPipelineRasterizationStateCreateInfo: {rasterization_state_info:#?}");

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let color_blend = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA,
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        };

        log::debug!("VkPipelineColorBlendAttachmentState: {color_blend:#?}");

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            attachment_count: 1,
            p_attachments: &color_blend as *const vk::PipelineColorBlendAttachmentState,
            ..Default::default()
        };

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
            ..Default::default()
        };

        let pipeline_layout = OnDropDefer::new(
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }
                .map_err(RendererError::FailedToCreatePipelineLayout)?,
            |p_layout| {
                log::debug!("Defered destroy pipeline layout called");
                unsafe { device.destroy_pipeline_layout(p_layout, None) };
            },
        );

        let pipeline_info = vk::GraphicsPipelineCreateInfo {
            stage_count: 2,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_stage_info as *const _,
            p_input_assembly_state: &input_assembly_stage_info as *const _,
            p_viewport_state: &viewport_state_info as *const _,
            p_rasterization_state: &rasterization_state_info as *const _,
            p_multisample_state: &multisample_state_info as *const _,
            p_depth_stencil_state: std::ptr::null(),
            p_color_blend_state: &color_blend_state as *const _,
            p_dynamic_state: &dynamic_state_info as *const _,
            layout: *pipeline_layout.as_ref(),
            render_pass,
            subpass: 0,
            ..Default::default()
        };

        let pipeline = *unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        }
        .map_err(|e| RendererError::FailedToCreateGraphicsPipeline(e.1))?
        .first()
        .unwrap();

        Ok((pipeline_layout.take(), pipeline))
    }

    fn create_shader_module(
        device: &ash::Device,
        code: &[u32],
    ) -> Result<vk::ShaderModule, RendererError> {
        let module_create_info = vk::ShaderModuleCreateInfo {
            // Size is given in bytes, so we multiply the length by 4
            code_size: code.len().checked_mul(4).unwrap().try_into().unwrap(),
            p_code: code.as_ptr(),
            ..Default::default()
        };

        unsafe { device.create_shader_module(&module_create_info, None) }
            .map_err(RendererError::FailedToCreateShaderModule)
    }

    fn record_command_buffer(
        &mut self,
        cmdbuf: vk::CommandBuffer,
        img_idx: usize,
    ) -> Result<(), RendererError> {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            ..Default::default()
        };

        unsafe {
            self.vk_res
                .device()
                .begin_command_buffer(cmdbuf, &command_buffer_begin_info)
        }
        .map_err(RendererError::CommandBufferRecordingError)?;

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            render_pass: self.vk_res.render_pass(),
            framebuffer: self.vk_res.framebuffers()[img_idx],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_img_extent,
            },
            clear_value_count: 1,
            p_clear_values: &vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            } as *const _,
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.swapchain_img_extent.width as f32,
            height: self.swapchain_img_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain_img_extent,
        };

        unsafe {
            self.vk_res.device().cmd_begin_render_pass(
                cmdbuf,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            self.vk_res.device().cmd_bind_pipeline(
                cmdbuf,
                vk::PipelineBindPoint::GRAPHICS,
                self.vk_res.pipeline(),
            );
            let buffers = [self.test_vertex_buffer.buffer_handle];
            self.vk_res.device().cmd_bind_vertex_buffers(
                cmdbuf,
                0,
                &buffers,
                &[self.test_vertex_buffer.offset],
            );
            self.vk_res.device().cmd_bind_index_buffer(
                cmdbuf,
                self.test_index_buffer.buffer_handle,
                self.test_index_buffer.offset,
                vk::IndexType::UINT16,
            );
            self.vk_res
                .device()
                .cmd_set_viewport(cmdbuf, 0, &[viewport]);
            self.vk_res.device().cmd_set_scissor(cmdbuf, 0, &[scissor]);
            self.vk_res
                .device()
                // .cmd_draw(cmdbuf, VERTICES.len().try_into().unwrap(), 1, 0, 0);
                .cmd_draw_indexed(cmdbuf, 6, 1, 0, 0, 0);

            self.vk_res.device().cmd_end_render_pass(cmdbuf);

            self.vk_res
                .device()
                .end_command_buffer(cmdbuf)
                .map_err(RendererError::CommandBufferRecordingError)
        }
    }

    pub fn draw_frame(&mut self) -> Result<(), RendererError> {
        let in_flight_fence = self.vk_res.in_flight_fences()[self.current_frame];

        let img_idx = unsafe {
            self.vk_res
                .device()
                // Wait for our fences
                .wait_for_fences(&[in_flight_fence], false, u64::MAX)
                .map_err(RendererError::FailedToDrawFrame)?;

            let (img_idx, _swapchain_suboptimal) = self
                .vk_res
                .swapchain_loader()
                // Acquire a new image to render to
                .acquire_next_image(
                    self.vk_res.swapchain(),
                    u64::MAX,
                    self.vk_res.img_available_semaphores()[self.current_frame],
                    vk::Fence::null(),
                )
                .map_err(RendererError::FailedToDrawFrame)?;

            self.vk_res
                .device()
                // Reset our command buffer in order to record our commands
                .reset_command_buffer(
                    self.command_buffers[self.current_frame],
                    vk::CommandBufferResetFlags::empty(),
                )
                .map_err(RendererError::FailedToDrawFrame)?;

            img_idx
        };

        // We call the our function that will record the command buffer
        self.record_command_buffer(
            self.command_buffers[self.current_frame],
            img_idx.try_into().unwrap(),
        )?;

        let wait_semaphores = [self.vk_res.img_available_semaphores()[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.vk_res.render_finished_semaphores()[self.current_frame]];
        let command_buffers = [self.command_buffers[self.current_frame]];
        let submit_info = vk::SubmitInfo {
            wait_semaphore_count: 1,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: command_buffers.as_ptr(),
            signal_semaphore_count: 1,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        unsafe {
            self.vk_res
                .device()
                .reset_fences(&[in_flight_fence])
                .map_err(RendererError::FailedToDrawFrame)?;

            self.vk_res
                .device()
                // Submit our commands to the graphics queue
                .queue_submit(
                    self.graphics_queue,
                    &[submit_info],
                    self.vk_res.in_flight_fences()[self.current_frame],
                )
                .map_err(RendererError::FailedToDrawFrame)?;

            let swapchains = [self.vk_res.swapchain()];

            let present_info = vk::PresentInfoKHR {
                wait_semaphore_count: 1,
                p_wait_semaphores: signal_semaphores.as_ptr(),
                swapchain_count: 1,
                p_swapchains: swapchains.as_ptr(),
                p_image_indices: &img_idx as *const u32,
                ..Default::default()
            };

            self.vk_res
                .swapchain_loader()
                .queue_present(self.present_queue, &present_info)
                .map_err(RendererError::FailedToDrawFrame)?;

            self.current_frame = (self.current_frame + 1) % MAX_CONCURRENT_FRAMES;
        }

        Ok(())
    }

    pub fn window_resized(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        log::debug!("Renderer resize requested: {width}, {height}");

        unsafe { self.vk_res.device().device_wait_idle() }.unwrap_or_else(|e| {
            log::error!("FATAL: Could not wait for device idle on window_resized: {e}");
            std::process::abort();
        });

        let swapchain_info = swapchain_info::SwapchainSupportInfo::fetch(
            self.vk_res.surface_loader(),
            self.vk_res.surface(),
            self.vk_res.physical_device(),
        )
        .map_err(RendererError::VulkanInfoQueryFailed)?;

        self.vk_res.destroy_swapchain();
        self.vk_res
            .create_swapchain(&swapchain_info, vk::Extent2D { width, height })?;

        self.swapchain_img_extent = swapchain_info.select_extent(vk::Extent2D { width, height });

        Ok(())
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
