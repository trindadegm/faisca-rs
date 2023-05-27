use ash::vk;

#[repr(transparent)]
pub struct Vector2(pub [f32; 2]);

#[repr(transparent)]
pub struct Vector3(pub [f32; 3]);

#[repr(transparent)]
pub struct Vector4(pub [f32; 4]);

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ComponentId(usize);

pub trait VertexComponent {
    fn id() -> ComponentId;
}

impl VertexComponent for f32 {
    #[inline(always)]
    fn id() -> ComponentId {
        ComponentId(1)
    }
}
impl VertexComponent for Vector2 {
    #[inline(always)]
    fn id() -> ComponentId {
        ComponentId(2)
    }
}
impl VertexComponent for Vector3 {
    #[inline(always)]
    fn id() -> ComponentId {
        ComponentId(3)
    }
}
impl VertexComponent for Vector4 {
    #[inline(always)]
    fn id() -> ComponentId {
        ComponentId(4)
    }
}

impl ComponentId {
    #[inline]
    pub fn vk_data(self) -> (vk::Format, u32) {
        match self.0 {
            1 => (vk::Format::R32_SFLOAT, 4),
            2 => (vk::Format::R32G32_SFLOAT, 8),
            3 => (vk::Format::R32G32B32_SFLOAT, 12),
            4 => (vk::Format::R32G32B32A32_SFLOAT, 16),
            _ => panic!("Invalid component Id. Probably a bug, maybe unsafe shenanigans?"),
        }
    }
}

pub struct VertexLayout {
    size: u32,
    components: Vec<ComponentId>,
}

impl VertexLayout {
    pub fn new() -> Self {
        VertexLayout {
            components: Vec::new(),
            size: 0,
        }
    }

    #[inline(always)]
    pub fn add_component<T: VertexComponent>(&mut self) {
        self.components.push(T::id());
        self.size = self.size
            .checked_add(T::id().vk_data().1)
            .expect("Vertex too big");
    }

    #[inline]
    pub fn num_components(&self) -> usize {
        self.components.len()
    }

    #[inline]
    pub fn vulkan_vertex_input_binding_description(
        &self,
        binding: u32,
        input_rate: vk::VertexInputRate,
    ) -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding,
            stride: self.size,
            input_rate,
        }
    }

    pub fn vulkan_describe_vertex_attributes(
        &self,
        binding: u32,
        out: &mut [vk::VertexInputAttributeDescription],
    ) {
        let mut offset = 0u32;
        for (index, &component) in self.components.iter().enumerate() {
            let (format, length) = component.vk_data();

            out[index] = vk::VertexInputAttributeDescription {
                binding,
                location: index.try_into().expect("Too many components"),
                format,
                offset,
            };

            offset = offset.checked_add(length).expect("Vertex is too big");
        }
    }
}

#[repr(C)]
pub struct Point2DColorRGBVertex {
    pub point: Vector2,
    pub color: Vector3,
}

impl Point2DColorRGBVertex {
    pub fn layout() -> VertexLayout {
        let mut layout = VertexLayout::new();
        layout.add_component::<Vector2>();
        layout.add_component::<Vector3>();
        layout
    }
}
