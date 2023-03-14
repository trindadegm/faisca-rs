use ash::{extensions::khr, vk};

#[derive(Default, Clone, Copy, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn fetch(
        instance: &ash::Instance,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        let mut queue_family_indices = QueueFamilyIndices::default();

        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        for (i, prop) in properties.iter().enumerate() {
            let i: u32 = i.try_into().unwrap();
            if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                queue_family_indices.graphics_family = Some(i)
            }

            queue_family_indices.present_family = unsafe {
                surface_loader.get_physical_device_surface_support(physical_device, i, surface)
            }
            .ok()
            .map(|_| i);

            if queue_family_indices.has_all() {
                break;
            }
        }

        queue_family_indices
    }

    pub fn has_all(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }

    #[inline]
    pub fn none() -> Self {
        Self {
            present_family: None,
            graphics_family: None,
        }
    }
}
