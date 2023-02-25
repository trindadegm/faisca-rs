use ash::vk;

#[derive(Default, Clone, Copy, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn fetch(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Self {
        let mut queue_family_indices = QueueFamilyIndices::default();

        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        for (i, prop) in properties.iter().enumerate() {
            if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                queue_family_indices.graphics_family = Some(i.try_into().unwrap())
            }
            if queue_family_indices.has_all() {
                break;
            }
        }

        queue_family_indices
    }

    pub fn has_all(&self) -> bool {
        self.graphics_family.is_some()
    }
}
