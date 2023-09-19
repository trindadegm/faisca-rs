use ash::{extensions::khr, vk};

#[derive(Default, Clone, Copy, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub dedicated_transfer_family: Option<u32>,
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

        let unified_graphics_and_present = properties.iter().enumerate().find(|&(i, prop)| {
            let i: u32 = i.try_into().unwrap();

            prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) && unsafe {
                surface_loader.get_physical_device_surface_support(physical_device, i, surface)
                    .unwrap_or(false)
            }
        })
        .map(|(i, _)| i.try_into().unwrap());

        if let Some(family) = unified_graphics_and_present {
            queue_family_indices.graphics_family = Some(family);
            queue_family_indices.present_family = Some(family);
        }

        queue_family_indices.dedicated_transfer_family = properties
            .iter()
            .enumerate()
            .find(|(_, prop)| {
                prop.queue_flags.contains(vk::QueueFlags::TRANSFER) &&
                (!prop.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            })
            .map(|(i, _)| i.try_into().unwrap());

        if !queue_family_indices.has_all() {
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
            dedicated_transfer_family: None,
        }
    }
}
