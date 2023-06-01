use crate::{renderer::RendererError, util::OnDropDefer};
use ash::vk;

use super::resources::RendererResourceKeeper;

const KIBIBYTE: vk::DeviceSize = 1024;
const MEBIBYTE: vk::DeviceSize = KIBIBYTE * KIBIBYTE;

const DEFAULT_VERTEX_BUFFER_SIZE: vk::DeviceSize = 64 * MEBIBYTE;

#[derive(Clone, Copy, Debug)]
struct AllocRecord {
    handle: vk::DeviceMemory,
    #[allow(unused)]
    size: vk::DeviceSize,
}

#[derive(Debug)]
struct BufferRecord {
    handle: vk::Buffer,
    alloc_idx: usize,
    size: vk::DeviceSize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferType {
    Vertex,
}

#[derive(Clone, Copy, Debug)]
pub struct VirtualBuffer {
    pub buffer_idx: usize,
    pub buffer_type: BufferType,
    pub buffer_handle: vk::Buffer,
    pub size: vk::DeviceSize,
    pub offset: vk::DeviceSize,
}

pub struct BufferManager {
    allocs: Vec<AllocRecord>,
    vertex_buffers: Vec<BufferRecord>,
    vertex_buffer_mem_properties: Vec<vk::MemoryRequirements>,
    vertex_buffer_tables: Vec<BufferAllocTable>,
    vertex_buffer_default_size: vk::DeviceSize,
}

struct BufferAllocTable {
    allocs: Vec<BufferAllocCell>,
    alignment_requirement: vk::DeviceSize,
}

struct BufferAllocCell {
    pub length: vk::DeviceSize,
    pub status: BufferAllocCellStatus,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BufferAllocCellStatus {
    Free,
    Occupied,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            allocs: Vec::new(),
            vertex_buffers: Vec::new(),
            vertex_buffer_mem_properties: Vec::new(),
            vertex_buffer_tables: Vec::new(),
            vertex_buffer_default_size: DEFAULT_VERTEX_BUFFER_SIZE,
        }
    }

    pub unsafe fn alloc_vertex_vbuffer(
        &mut self,
        vk_res: &RendererResourceKeeper,
        vbuffer_size: vk::DeviceSize,
    ) -> Result<VirtualBuffer, RendererError> {
        self.vertex_buffer_tables
            .iter_mut()
            .enumerate()
            .find_map(|(index, table)| table.try_fit(vbuffer_size).map(|o| (index, o)))
            .map_or_else(
                || {
                    let index =
                        self.create_new_vertex_buffer(vk_res, self.vertex_buffer_default_size)?;
                    self.vertex_buffer_tables.push(BufferAllocTable::new(
                        self.vertex_buffers[index].size,
                        self.vertex_buffer_mem_properties[index].alignment,
                    ));
                    self.vertex_buffer_tables[index]
                        .try_fit(vbuffer_size)
                        .ok_or_else(|| RendererError::ObjectTooBig)
                        .map(|offset| (index, offset))
                },
                Ok,
            )
            .map(|(index, offset)| VirtualBuffer {
                buffer_idx: index,
                buffer_type: BufferType::Vertex,
                size: vbuffer_size,
                buffer_handle: self.vertex_buffers[index].handle,
                offset,
            })
    }

    /// Returns the created buffer index
    unsafe fn create_new_vertex_buffer(
        &mut self,
        vk_res: &RendererResourceKeeper,
        size: vk::DeviceSize,
    ) -> Result<usize, RendererError> {
        let instance = vk_res.instance();
        let device = vk_res.device();
        let physical_device = vk_res.physical_device();

        let buffer_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = device
            .create_buffer(&buffer_info, None)
            .map_err(RendererError::FailedToCreateBuffer)?;
        let buffer_defer = OnDropDefer::new(buffer, |b| {
            log::info!("Deferred drop of buffer");
            device.destroy_buffer(b, None);
        });

        let buffer_idx = self.vertex_buffers.len();

        let buffer_mem_requirements = device.get_buffer_memory_requirements(buffer);
        let memory_properties = instance.get_physical_device_memory_properties(physical_device);
        let memory_property_flags =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let memory_type_index = Self::find_memory_type(
            memory_properties,
            buffer_mem_requirements.memory_type_bits,
            memory_property_flags,
        )
        .ok_or(RendererError::UnavailableMemoryType {
            memory_type_flags: buffer_mem_requirements.memory_type_bits,
            memory_property_flags,
        })?;

        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: size,
            memory_type_index,
            ..Default::default()
        };

        let device_memory = device
            .allocate_memory(&alloc_info, None)
            .map_err(RendererError::MemAllocError)?;
        let device_memory_defer = OnDropDefer::new(device_memory, |dm| {
            log::info!("Deferred drop of allocated device memory");
            device.free_memory(dm, None);
        });

        vk_res
            .device()
            .bind_buffer_memory(buffer, device_memory, 0)
            .map_err(RendererError::FailedToCreateBuffer)?;

        let alloc_idx = self.allocs.len();

        self.vertex_buffers.push(BufferRecord {
            handle: buffer_defer.take(),
            alloc_idx: 0,
            size,
        });
        self.allocs.push(AllocRecord {
            handle: device_memory_defer.take(),
            size,
        });
        self.vertex_buffer_mem_properties
            .push(buffer_mem_requirements);

        self.vertex_buffers[buffer_idx].alloc_idx = alloc_idx;

        Ok(buffer_idx)
    }

    fn find_memory_type(
        mem_properties: vk::PhysicalDeviceMemoryProperties,
        type_filter: u32,
        property_flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        for i in 0..mem_properties.memory_type_count {
            let idx: usize = i.try_into().unwrap();
            if (type_filter & (1 << i) != 0)
                && (mem_properties.memory_types[idx].property_flags & property_flags
                    == property_flags)
            {
                return Some(i);
            }
        }
        None
    }

    pub unsafe fn direct_upload(
        &mut self,
        vk_res: &RendererResourceKeeper,
        vbuffer: &VirtualBuffer,
        data: &[u8],
    ) -> Result<(), RendererError> {
        let size: vk::DeviceSize = data.len().try_into().unwrap();

        if size > vbuffer.size {
            return Err(RendererError::ObjectTooBig);
        }

        let alloc_record_idx = self.get_underlying_alloc_idx(vbuffer);
        let alloc_record = self.allocs[alloc_record_idx];

        // let buffer = self.buffers
        let data_ptr = vk_res
            .device()
            .map_memory(
                alloc_record.handle,
                vbuffer.offset,
                size,
                vk::MemoryMapFlags::empty(),
            )
            .map_err(RendererError::FailedToMapBufferMemory)?;
        unsafe {
            let data_slice = std::slice::from_raw_parts_mut(data_ptr as *mut u8, data.len());
            data_slice.copy_from_slice(data);
        }

        vk_res.device().unmap_memory(alloc_record.handle);

        Ok(())
    }

    fn get_underlying_alloc_idx(&self, vbuffer: &VirtualBuffer) -> usize {
        use BufferType::*;
        match vbuffer.buffer_type {
            Vertex => self.vertex_buffers[vbuffer.buffer_idx].alloc_idx,
        }
    }

    /// Destroy all the buffers and frees all of the memory.
    ///
    /// # Safety
    /// Make sure the resources are not in use and that they were created with
    /// the same device.
    pub unsafe fn destroy(&mut self, device: &mut ash::Device) {
        log::debug!(
            "Destroying {n} Vulkan buffers",
            n = self.vertex_buffers.len()
        );
        for buffer in self.vertex_buffers.drain(..) {
            device.destroy_buffer(buffer.handle, None);
        }

        log::debug!(
            "Freeing {n} Vulkan memory allocations",
            n = self.allocs.len()
        );
        for alloc in self.allocs.drain(..) {
            device.free_memory(alloc.handle, None);
        }
    }
}
impl Default for BufferManager {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl BufferAllocTable {
    pub fn new(size: vk::DeviceSize, alignment_requirement: vk::DeviceSize) -> Self {
        Self {
            allocs: vec![BufferAllocCell {
                length: size,
                status: BufferAllocCellStatus::Free,
            }],
            alignment_requirement,
        }
    }

    pub fn try_fit(&mut self, mut req_size: vk::DeviceSize) -> Option<vk::DeviceSize> {
        // Guarantee alignemnt of the next block by increasing this one's size
        req_size += req_size % self.alignment_requirement;

        let mut offset: vk::DeviceSize = 0;
        for (index, cell) in self.allocs.iter_mut().enumerate() {
            if cell.status == BufferAllocCellStatus::Free {
                if cell.length >= req_size {
                    cell.status = BufferAllocCellStatus::Occupied;
                    let remaining_length = cell.length - req_size;
                    if remaining_length > 0 {
                        let new_cell = BufferAllocCell {
                            length: remaining_length,
                            status: BufferAllocCellStatus::Free,
                        };
                        self.allocs.insert(index.checked_add(1).unwrap(), new_cell);
                    }

                    return Some(offset);
                }
            }

            offset = offset.checked_add(cell.length).unwrap();
        }

        None
    }
}
