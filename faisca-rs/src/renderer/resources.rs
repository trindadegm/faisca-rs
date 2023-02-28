use ash::{extensions::{ext, khr}, vk};

pub struct RendererResourceKeeper {
    instance: Option<ash::Instance>,
    debug_loader: Option<ext::DebugUtils>,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    surface_loader: Option<khr::Surface>,
    surface: vk::SurfaceKHR,

    device: Option<ash::Device>,

    swapchain_loader: Option<khr::Swapchain>,
    swapchain: vk::SwapchainKHR,
    swapchain_image_views: Vec<vk::ImageView>,

    render_pass: vk::RenderPass,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    framebuffers: Vec<vk::Framebuffer>,

    command_pool: vk::CommandPool,

    img_available_sem: vk::Semaphore,
    render_finished_sem: vk::Semaphore,
    in_flight_fence: vk::Fence,
}

impl RendererResourceKeeper {
    pub fn new() -> Self {
        Default::default()
    }
}
impl Default for RendererResourceKeeper {
    fn default() -> Self {
        Self {
            instance: None,
            debug_loader: None,
            debug_messenger: vk::DebugUtilsMessengerEXT::null(),

            surface_loader: None,
            surface: vk::SurfaceKHR::null(),

            device: None,

            swapchain_loader: None,
            swapchain: vk::SwapchainKHR::null(),
            swapchain_image_views: Vec::new(),

            render_pass: vk::RenderPass::null(),

            pipeline_layout: vk::PipelineLayout::null(),
            pipeline: vk::Pipeline::null(),

            framebuffers: Vec::new(),

            command_pool: vk::CommandPool::null(),

            img_available_sem: vk::Semaphore::null(),
            render_finished_sem: vk::Semaphore::null(),
            in_flight_fence: vk::Fence::null(),
        }
    }
}
impl Drop for RendererResourceKeeper {
    fn drop(&mut self) {
        if let Some(device) = &self.device {
            unsafe { device.device_wait_idle() }
                .unwrap_or_else(|e| {
                    log::error!("FATAL: Could not wait for device idle on Renderer destroying: {e}");
                    std::process::abort();
                });

            log::debug!("Destroying Vulkan image available semaphore");
            unsafe { device.destroy_semaphore(self.img_available_sem, None) };

            log::debug!("Destroying Vulkan render finished semaphore");
            unsafe { device.destroy_semaphore(self.render_finished_sem, None) };

            log::debug!("Destroying Vulkan in flight fence");
            unsafe { device.destroy_fence(self.in_flight_fence, None) };

            log::debug!("Destroying Vulkan command pool");
            unsafe { device.destroy_command_pool(self.command_pool, None) };

            log::debug!("Destroying Vulkan framebuffers");
            for &fbuf in self.framebuffers.iter() {
                unsafe { device.destroy_framebuffer(fbuf, None) };
            }

            log::debug!("Destroying Vulkan pipeline");
            unsafe { device.destroy_pipeline(self.pipeline, None) };

            log::debug!("Destroying Vulkan pipeline layout");
            unsafe {
                device
                    .destroy_pipeline_layout(self.pipeline_layout, None)
            };

            log::debug!("Destroying Vulkan render pass");
            unsafe { device.destroy_render_pass(self.render_pass, None) };

            log::debug!("Destroying Vulkan image views");
            for &view in self.swapchain_image_views.iter() {
                unsafe { device.destroy_image_view(view, None) };
            }

            if let Some(swapchain_loader) = &self.swapchain_loader {
                log::debug!("Destroying Vulkan swapchain");
                unsafe {
                    swapchain_loader.destroy_swapchain(self.swapchain, None)
                };
            }

            log::debug!("Destroying Vulkan device");
            unsafe { device.destroy_device(None) };
        }

        if let Some(surface_loader) = &self.surface_loader {
            log::debug!("Destroying Vulkan surface");
            unsafe { surface_loader.destroy_surface(self.surface, None) };
        }

        if let Some(debug_loader) = &self.debug_loader {
            log::debug!("Destroying Vulkan debug utils");
            unsafe { debug_loader.destroy_debug_utils_messenger(self.debug_messenger, None) };
        }

        if let Some(instance) = &self.instance {
            log::debug!("Destroying Vulkan instance");
            unsafe { instance.destroy_instance(None) };
        }
    }
}
