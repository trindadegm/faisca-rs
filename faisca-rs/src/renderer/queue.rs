use ash::vk;

#[derive(Default, Clone, Copy, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn fetch(
        entry: &ash::Entry,
        instance: &ash::Instance,
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

            let surface_ext = ash::extensions::khr::Surface::new(entry, instance);
            queue_family_indices.present_family = unsafe {
                surface_ext.get_physical_device_surface_support(physical_device, i, surface)
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
}
