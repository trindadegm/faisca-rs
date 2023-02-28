#![allow(unused)]

use ash::{extensions::{ext, khr}, vk};
use crate::renderer::RendererError;

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

    img_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
}

impl RendererResourceKeeper {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn instance(&self) -> &ash::Instance {
        self.instance.as_ref().unwrap()
    }

    #[inline]
    pub unsafe fn instance_mut(&mut self) -> &mut Option<ash::Instance> {
        &mut self.instance
    }

    #[inline]
    pub fn debug_loader(&self) -> &ext::DebugUtils {
        self.debug_loader.as_ref().unwrap()
    }

    #[inline]
    pub unsafe fn debug_loader_mut(&mut self) -> &mut Option<ext::DebugUtils> {
        &mut self.debug_loader
    }

    #[inline]
    pub fn debug_messenger(&self) -> vk::DebugUtilsMessengerEXT {
        self.debug_messenger
    }

    #[inline]
    pub unsafe fn debug_messenger_mut(&mut self) -> &mut vk::DebugUtilsMessengerEXT {
        &mut self.debug_messenger
    }

    #[inline]
    pub fn surface_loader(&self) -> &khr::Surface {
        self.surface_loader.as_ref().unwrap()
    }

    #[inline]
    pub unsafe fn surface_loader_mut(&mut self) -> &mut Option<khr::Surface> {
        &mut self.surface_loader
    }

    #[inline]
    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    #[inline]
    pub unsafe fn surface_mut(&mut self) -> &mut vk::SurfaceKHR {
        &mut self.surface
    }

    #[inline]
    pub fn device(&self) -> &ash::Device {
        self.device.as_ref().unwrap()
    }

    #[inline]
    pub unsafe fn device_mut(&mut self) -> &mut Option<ash::Device> {
        &mut self.device
    }

    #[inline]
    pub fn swapchain_loader(&self) -> &khr::Swapchain {
        self.swapchain_loader.as_ref().unwrap()
    }

    #[inline]
    pub unsafe fn swapchain_loader_mut(&mut self) -> &mut Option<khr::Swapchain> {
        &mut self.swapchain_loader
    }

    #[inline]
    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    #[inline]
    pub unsafe fn swapchain_mut(&mut self) -> &mut vk::SwapchainKHR {
        &mut self.swapchain
    }

    #[inline]
    pub fn swapchain_image_views(&self) -> &[vk::ImageView] {
        self.swapchain_image_views.as_slice()
    }

    #[inline]
    pub unsafe fn swapchain_image_views_mut(&mut self) -> &mut Vec<vk::ImageView> {
        &mut self.swapchain_image_views
    }

    #[inline]
    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    #[inline]
    pub unsafe fn render_pass_mut(&mut self) -> &mut vk::RenderPass {
        &mut self.render_pass
    }

    #[inline]
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    #[inline]
    pub unsafe fn pipeline_layout_mut(&mut self) -> &mut vk::PipelineLayout {
        &mut self.pipeline_layout
    }

    #[inline]
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    #[inline]
    pub unsafe fn pipeline_mut(&mut self) -> &mut vk::Pipeline {
        &mut self.pipeline
    }

    #[inline]
    pub fn framebuffers(&self) -> &[vk::Framebuffer] {
        self.framebuffers.as_slice()
    }

    #[inline]
    pub unsafe fn framebuffers_mut(&mut self) -> &mut Vec<vk::Framebuffer> {
        &mut self.framebuffers
    }

    #[inline]
    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }

    #[inline]
    pub unsafe fn command_pool_mut(&mut self) -> &mut vk::CommandPool {
        &mut self.command_pool
    }

    #[inline]
    pub fn img_available_semaphores(&self) -> &[vk::Semaphore] {
        &self.img_available_semaphores
    }

    #[inline]
    pub unsafe fn img_available_semaphores_mut(&mut self) -> &mut Vec<vk::Semaphore> {
        &mut self.img_available_semaphores
    }

    #[inline]
    pub fn render_finished_semaphores(&self) -> &[vk::Semaphore] {
        &self.render_finished_semaphores
    }

    #[inline]
    pub unsafe fn render_finished_semaphores_mut(&mut self) -> &mut Vec<vk::Semaphore> {
        &mut self.render_finished_semaphores
    }

    #[inline]
    pub fn in_flight_fences(&self) -> &[vk::Fence] {
        &self.in_flight_fences
    }

    #[inline]
    pub unsafe fn in_flight_fences_mut(&mut self) -> &mut Vec<vk::Fence> {
        &mut self.in_flight_fences
    }

    pub unsafe fn create_sync_objects(&mut self, count: usize) -> Result<(), RendererError> {
        self.img_available_semaphores.reserve(count);
        self.render_finished_semaphores.reserve(count);
        self.in_flight_fences.reserve(count);

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        for i in 0..count {
            self.img_available_semaphores.push(
                self.device().create_semaphore(&semaphore_info, None)
                    .map_err(RendererError::FailedToCreateSyncObject)?
            );
            self.render_finished_semaphores.push(
                self.device().create_semaphore(&semaphore_info, None)
                    .map_err(RendererError::FailedToCreateSyncObject)?
            );
            self.in_flight_fences.push(
                self.device().create_fence(&fence_info, None)
                    .map_err(RendererError::FailedToCreateSyncObject)?
            );
        }

        Ok(())
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

            img_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            in_flight_fences: Vec::new(),
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

            log::debug!("Destroying Vulkan semaphores");
            for &sem in self.img_available_semaphores.iter().chain(self.render_finished_semaphores.iter()) {
                unsafe { device.destroy_semaphore(sem, None) };
            }

            log::debug!("Destroying Vulkan fences");
            for &fence in self.in_flight_fences.iter() {
                unsafe { device.destroy_fence(fence, None) };
            }

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
