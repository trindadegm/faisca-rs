use ash::vk;

#[repr(C, align(16))]
pub struct Mat4([f32; 16]);

#[repr(C)]
pub struct StandardUBO {
    pub model: Mat4,
    pub view: Mat4,
}

impl Mat4 {
    #[inline]
    pub fn new(data: [f32; 16]) -> Self {
        Self(data)
    }

    #[rustfmt::skip]
    pub fn identity() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[inline]
    pub fn data(&self) -> &[f32; 16] {
        &self.0
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [f32; 16] {
        &mut self.0
    }
}

impl StandardUBO {
    pub fn uniform_buffer_binding(binding: u32) -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding {
            binding,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        }
    }
}
impl Default for StandardUBO {
    fn default() -> Self {
        Self {
            model: Mat4::identity(),
            view: Mat4::identity(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubo_layout_test() {
        use std::mem;

        assert_eq!(mem::size_of::<StandardUBO>(), 16 * 4 * 3);
        assert_eq!(mem::align_of::<StandardUBO>(), 16);

        let something = StandardUBO {
            model: Mat4::identity(),
            view: Mat4::identity(),
        };

        let something_ptr = &something as *const StandardUBO as usize;
        let model_ptr = &something.model as *const Mat4 as usize;
        let view_ptr = &something.view as *const Mat4 as usize;

        assert_eq!(something_ptr, model_ptr);
        assert_eq!(something_ptr + 16 * 4, view_ptr);
    }
}
