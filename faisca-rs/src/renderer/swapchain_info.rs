use ash::{extensions::khr, prelude::VkResult, vk};

#[derive(Clone)]
pub struct SwapchainSupportInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupportInfo {
    pub fn fetch(
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> VkResult<Self> {
        let capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
        };
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)?
        };
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
        };

        Ok(Self {
            capabilities,
            formats,
            present_modes,
        })
    }

    pub fn select_format(&self) -> Option<vk::SurfaceFormatKHR> {
        self.formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_SRGB
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .or_else(|| self.formats.first().copied())
    }

    pub fn select_present_mode(&self) -> vk::PresentModeKHR {
        self.present_modes
            .iter()
            .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
            .copied()
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    pub fn select_extent(&self, window_extent: vk::Extent2D) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            self.capabilities.current_extent
        } else {
            // In case it is set to u32::MAX, it gives us leeway into figuring
            // an extent, this is where window_extent comes useful. It makes
            // sense getting an extent similar to the window size.
            let extent = vk::Extent2D {
                width: window_extent.width.clamp(
                    self.capabilities.min_image_extent.width,
                    self.capabilities.max_image_extent.width,
                ),
                height: window_extent.height.clamp(
                    self.capabilities.min_image_extent.height,
                    self.capabilities.max_image_extent.height,
                ),
            };
            extent
        }
    }
}
