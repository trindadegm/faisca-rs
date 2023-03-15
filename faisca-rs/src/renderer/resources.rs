#![allow(unused)]

use crate::{
    ffi,
    renderer::{queue::QueueFamilyIndices, swapchain_info::SwapchainSupportInfo, RendererError},
};
use ash::{
    extensions::{ext, khr},
    vk,
};

pub struct RendererResourceKeeper {
    instance: Option<ash::Instance>,
    debug_loader: Option<ext::DebugUtils>,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    surface_loader: Option<khr::Surface>,
    surface: vk::SurfaceKHR,

    physical_device: vk::PhysicalDevice,

    queue_families: QueueFamilyIndices,

    device: Option<ash::Device>,

    swapchain_loader: Option<khr::Swapchain>,
    swapchain: vk::SwapchainKHR,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_info: Option<SwapchainSupportInfo>,

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
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    #[inline]
    pub unsafe fn physical_device_mut(&mut self) -> &mut vk::PhysicalDevice {
        &mut self.physical_device
    }

    #[inline]
    pub fn queue_families(&self) -> &QueueFamilyIndices {
        &self.queue_families
    }

    #[inline]
    pub unsafe fn queue_families_mut(&mut self) -> &mut QueueFamilyIndices {
        &mut self.queue_families
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
    pub fn swapchain_info(&self) -> &SwapchainSupportInfo {
        self.swapchain_info.as_ref().unwrap()
    }

    #[inline]
    pub fn swapchain_info_mut(&mut self) -> &mut SwapchainSupportInfo {
        self.swapchain_info.as_mut().unwrap()
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
                self.device()
                    .create_semaphore(&semaphore_info, None)
                    .map_err(RendererError::FailedToCreateSyncObject)?,
            );
            self.render_finished_semaphores.push(
                self.device()
                    .create_semaphore(&semaphore_info, None)
                    .map_err(RendererError::FailedToCreateSyncObject)?,
            );
            self.in_flight_fences.push(
                self.device()
                    .create_fence(&fence_info, None)
                    .map_err(RendererError::FailedToCreateSyncObject)?,
            );
        }

        Ok(())
    }

    pub fn create_swapchain(
        &mut self,
        swapchain_info: &SwapchainSupportInfo,
        window_extent: vk::Extent2D,
    ) -> Result<(), RendererError> {
        log::debug!("Creating swapchain with window extent: {window_extent:?}");
        log::debug!("Swapchain info: {swapchain_info:#?}");

        let surface_format = swapchain_info.select_format().unwrap();
        let present_mode = swapchain_info.select_present_mode();

        let selected_extent = swapchain_info.select_extent(window_extent);

        let image_count = swapchain_info.capabilities.min_image_count
            + if swapchain_info.capabilities.min_image_count
                != swapchain_info.capabilities.max_image_count
            {
                // If the number of images is not set in stone to be that one, we pick one more
                1
            } else {
                // Otherwise we keep the required one, (add 0 does nothing)
                0
            };

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
            surface: self.surface(),
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: selected_extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            pre_transform: swapchain_info.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            ..Default::default()
        };

        let queue_family_indices = [
            self.queue_families().graphics_family.unwrap(),
            self.queue_families().present_family.unwrap(),
        ];
        if self.queue_families().graphics_family.unwrap()
            != self.queue_families().present_family.unwrap()
        {
            swapchain_create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            swapchain_create_info.queue_family_index_count = 2;
            swapchain_create_info.p_queue_family_indices = queue_family_indices.as_ptr();
        } else {
            swapchain_create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        }

        self.swapchain = unsafe {
            self.swapchain_loader()
                .create_swapchain(&swapchain_create_info, None)
                .map_err(RendererError::FailedToCreateSwapchain)?
        };

        self.swapchain_info = Some(swapchain_info.clone());

        log::debug!("Creating image views");
        self.create_image_views(surface_format)?;
        log::debug!("Creating framebuffers");
        self.create_framebuffers(selected_extent)?;

        Ok(())
    }

    fn create_image_views(
        &mut self,
        swapchain_img_format: vk::SurfaceFormatKHR,
    ) -> Result<(), RendererError> {
        let images = unsafe {
            self.swapchain_loader()
                .get_swapchain_images(self.swapchain())
                .map_err(RendererError::FailedToCreateImageView)?
        };

        assert!(self.swapchain_image_views.is_empty());
        for image in images {
            let img_view_info = vk::ImageViewCreateInfo {
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: swapchain_img_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };

            let image_view = unsafe { self.device().create_image_view(&img_view_info, None) }
                .map_err(RendererError::FailedToCreateImageView)?;

            self.swapchain_image_views.push(image_view);
        }

        Ok(())
    }

    fn create_framebuffers(&mut self, extent: vk::Extent2D) -> Result<(), RendererError> {
        assert!(self.framebuffers.is_empty());
        for image_view in &self.swapchain_image_views {
            let framebuffer_info = vk::FramebufferCreateInfo {
                render_pass: self.render_pass(),
                attachment_count: 1,
                p_attachments: image_view as *const _,
                width: extent.width,
                height: extent.height,
                layers: 1,
                ..Default::default()
            };

            let framebuffer = unsafe { self.device().create_framebuffer(&framebuffer_info, None) }
                .map_err(RendererError::FailedToCreateFramebuffer)?;

            self.framebuffers.push(framebuffer);
        }

        Ok(())
    }

    pub fn destroy_swapchain(&mut self) {
        log::debug!("Destroying Vulkan framebuffers");
        for &fbuf in self.framebuffers.iter() {
            unsafe { self.device().destroy_framebuffer(fbuf, None) };
        }
        self.framebuffers.clear();

        log::debug!("Destroying Vulkan image views");
        for &view in self.swapchain_image_views.iter() {
            unsafe { self.device().destroy_image_view(view, None) };
        }
        self.swapchain_image_views.clear();

        if let Some(swapchain_loader) = &self.swapchain_loader {
            log::debug!("Destroying Vulkan swapchain");
            unsafe { swapchain_loader.destroy_swapchain(self.swapchain, None) };
        }
    }

    pub fn recreate_swapchain(&mut self) -> Result<(), RendererError> {
        unsafe { self.device().device_wait_idle() };

        // unsafe { self.destroy_swapchain() };
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

            physical_device: vk::PhysicalDevice::null(),
            queue_families: QueueFamilyIndices::none(),

            device: None,

            swapchain_loader: None,
            swapchain: vk::SwapchainKHR::null(),
            swapchain_image_views: Vec::new(),
            swapchain_info: None,

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
        // if let Some(device) = &self.device {
        if self.device.is_some() {
            unsafe { self.device().device_wait_idle() }.unwrap_or_else(|e| {
                log::error!("FATAL: Could not wait for device idle on Renderer destroying: {e}");
                std::process::abort();
            });

            log::debug!("Destroying Vulkan semaphores");
            for &sem in self
                .img_available_semaphores
                .iter()
                .chain(self.render_finished_semaphores.iter())
            {
                unsafe { self.device().destroy_semaphore(sem, None) };
            }

            log::debug!("Destroying Vulkan fences");
            for &fence in self.in_flight_fences.iter() {
                unsafe { self.device().destroy_fence(fence, None) };
            }

            log::debug!("Destroying Vulkan command pool");
            unsafe { self.device().destroy_command_pool(self.command_pool, None) };

            log::debug!("Destroying Vulkan pipeline");
            unsafe { self.device().destroy_pipeline(self.pipeline, None) };

            log::debug!("Destroying Vulkan pipeline layout");
            unsafe {
                self.device()
                    .destroy_pipeline_layout(self.pipeline_layout, None)
            };

            log::debug!("Destroying Vulkan render pass");
            unsafe { self.device().destroy_render_pass(self.render_pass, None) };

            self.destroy_swapchain();

            log::debug!("Destroying Vulkan device");
            unsafe { self.device().destroy_device(None) };
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
