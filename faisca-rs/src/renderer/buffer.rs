use std::collections::HashMap;

use crate::{renderer::RendererError, util::OnDropDefer};
use ash::vk;

use super::resources::RendererResourceKeeper;

const KIBIBYTE: vk::DeviceSize = 1024;
const MEBIBYTE: vk::DeviceSize = KIBIBYTE * KIBIBYTE;

const DEFAULT_VERTEX_BUFFER_SIZE: vk::DeviceSize = 64 * MEBIBYTE;
const DEFAULT_STAGING_BUFFER_SIZE: vk::DeviceSize = 64 * MEBIBYTE;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum BufferType {
    Staging,
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

struct SpecificBufferManager {
    buffers: Vec<BufferRecord>,
    buffer_mem_properties: Vec<vk::MemoryRequirements>,
    buffer_tables: Vec<BufferAllocTable>,
    buffer_default_size: vk::DeviceSize,
}

pub struct BufferManager {
    allocs: Vec<AllocRecord>,

    specific_managers: HashMap<BufferType, SpecificBufferManager>,
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

            specific_managers: HashMap::default(),
        }
    }

    pub unsafe fn alloc_staging_vbuffer(
        &mut self,
        vk_res: &RendererResourceKeeper,
        vbuffer_size: vk::DeviceSize,
    ) -> Result<VirtualBuffer, RendererError> {
        self.alloc_vbuffer(vk_res, vbuffer_size, BufferType::Staging)
    }

    pub unsafe fn alloc_vertex_vbuffer(
        &mut self,
        vk_res: &RendererResourceKeeper,
        vbuffer_size: vk::DeviceSize,
    ) -> Result<VirtualBuffer, RendererError> {
        self.alloc_vbuffer(vk_res, vbuffer_size, BufferType::Vertex)
    }

    unsafe fn alloc_vbuffer(
        &mut self,
        vk_res: &RendererResourceKeeper,
        vbuffer_size: vk::DeviceSize,
        buffer_type: BufferType,
    ) -> Result<VirtualBuffer, RendererError> {
        let sm = self
            .specific_managers
            .entry(buffer_type)
            .or_insert(SpecificBufferManager {
                buffers: Vec::new(),
                buffer_mem_properties: Vec::new(),
                buffer_tables: Vec::new(),
                buffer_default_size: buffer_type.default_size(),
            });

        sm.buffer_tables
            .iter_mut()
            .enumerate()
            .find_map(|(index, table)| table.try_fit(vbuffer_size).map(|o| (index, o)))
            .map_or_else(
                || {
                    let index = Self::create_new_buffer(
                        sm,
                        &mut self.allocs,
                        vk_res,
                        sm.buffer_default_size,
                        buffer_type,
                    )?;
                    sm.buffer_tables.push(BufferAllocTable::new(
                        sm.buffers[index].size,
                        sm.buffer_mem_properties[index].alignment,
                    ));
                    sm.buffer_tables[index]
                        .try_fit(vbuffer_size)
                        .ok_or_else(|| RendererError::ObjectTooBig)
                        .map(|offset| (index, offset))
                },
                Ok,
            )
            .map(|(index, offset)| VirtualBuffer {
                buffer_idx: index,
                buffer_type,
                size: vbuffer_size,
                buffer_handle: sm.buffers[index].handle,
                offset,
            })
    }

    /// Returns the created buffer index
    unsafe fn create_new_buffer(
        sm: &mut SpecificBufferManager,
        allocs: &mut Vec<AllocRecord>,
        vk_res: &RendererResourceKeeper,
        size: vk::DeviceSize,
        buffer_type: BufferType,
    ) -> Result<usize, RendererError> {
        let instance = vk_res.instance();
        let device = vk_res.device();
        let physical_device = vk_res.physical_device();

        let buffer_info = vk::BufferCreateInfo {
            size,
            usage: Self::buffer_type_usage(buffer_type),
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

        let buffer_idx = sm.buffers.len();

        let buffer_mem_requirements = device.get_buffer_memory_requirements(buffer);
        let memory_properties = instance.get_physical_device_memory_properties(physical_device);
        let memory_property_flags = Self::buffer_type_mem_props(buffer_type);

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

        let alloc_idx = allocs.len();

        sm.buffers.push(BufferRecord {
            handle: buffer_defer.take(),
            alloc_idx: 0,
            size,
        });
        allocs.push(AllocRecord {
            handle: device_memory_defer.take(),
            size,
        });
        sm.buffer_mem_properties.push(buffer_mem_requirements);

        sm.buffers[buffer_idx].alloc_idx = alloc_idx;

        Ok(buffer_idx)
    }

    fn buffer_type_usage(buffer_type: BufferType) -> vk::BufferUsageFlags {
        match buffer_type {
            BufferType::Vertex => {
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST
            }
            BufferType::Staging => vk::BufferUsageFlags::TRANSFER_SRC,
        }
    }

    fn buffer_type_mem_props(buffer_type: BufferType) -> vk::MemoryPropertyFlags {
        match buffer_type {
            BufferType::Vertex => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            BufferType::Staging => {
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
            }
        }
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
        self.specific_managers
            .get(&vbuffer.buffer_type)
            .map(|sm| sm.buffers[vbuffer.buffer_idx].alloc_idx)
            .unwrap()
    }

    /// Destroy all the buffers and frees all of the memory.
    ///
    /// # Safety
    /// Make sure the resources are not in use and that they were created with
    /// the same device.
    pub unsafe fn destroy(&mut self, device: &mut ash::Device) {
        for sm in self.specific_managers.values_mut() {
            log::debug!("Destroying {n} Vulkan buffers", n = sm.buffers.len());
            for buffer in sm.buffers.drain(..) {
                device.destroy_buffer(buffer.handle, None);
            }
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
        let align_remainder = req_size % self.alignment_requirement;
        if align_remainder > 0 {
            req_size += self.alignment_requirement - align_remainder;
        }

        let mut offset: vk::DeviceSize = 0;
        for (index, cell) in self.allocs.iter_mut().enumerate() {
            if cell.status == BufferAllocCellStatus::Free {
                if cell.length >= req_size {
                    let remaining_length = cell.length - req_size;

                    cell.status = BufferAllocCellStatus::Occupied;
                    cell.length = req_size;

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

    pub fn free(&mut self, position: vk::DeviceSize) {
        let mut offset: vk::DeviceSize = 0;
        for index in 0..self.allocs.len() {
            if position == offset && self.allocs[index].status == BufferAllocCellStatus::Occupied {
                self.allocs[index].status = BufferAllocCellStatus::Free;

                self.collapse_around(index);

                return;
            }

            offset = offset.checked_add(self.allocs[index].length).unwrap();
        }
    }

    fn collapse_around(&mut self, index: usize) {
        let mut length: vk::DeviceSize = 0;
        let mut n_to_collapse = 0usize;
        let mut first_idx = index;

        index.checked_add(1)
            .and_then(|i| {
                self.allocs.get(i)
                    .map(|c| {
                        if c.status == BufferAllocCellStatus::Free {
                            n_to_collapse += 1;
                            length = length.checked_add(c.length).unwrap();
                        }
                    })
            });

        self.allocs.get(index)
            .map(|c| length = length.checked_add(c.length).unwrap());

        index.checked_sub(1)
            .and_then(|i| {
                self.allocs.get(i)
                    .map(|c| {
                        if c.status == BufferAllocCellStatus::Free {
                            n_to_collapse += 1;
                            first_idx = i;
                            length = length.checked_add(c.length).unwrap();
                        }
                    })
            });

        self.allocs[first_idx].length = length;
        for delta_idx in (1..=n_to_collapse).rev() {
            let idx_to_remove = first_idx + delta_idx;
            self.allocs.remove(idx_to_remove);
        }
    }
}

impl BufferType {
    pub fn default_size(self) -> vk::DeviceSize {
        use BufferType::*;
        match self {
            Staging => DEFAULT_STAGING_BUFFER_SIZE,
            Vertex => DEFAULT_VERTEX_BUFFER_SIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_table_test() {
        let size = 256;
        let alignment = 8;
        let mut alloc_table = BufferAllocTable::new(size, alignment);

        assert!(alloc_table.try_fit(257).is_none());
        assert_eq!(alloc_table.allocs[0].status, BufferAllocCellStatus::Free);
        assert_eq!(alloc_table.allocs[0].length, 256);

        assert_eq!(alloc_table.try_fit(256), Some(0));

        alloc_table.free(0);

        assert_eq!(alloc_table.allocs.iter().map(|c| c.length).fold(0, |x, a| x + a), 256);

        assert_eq!(alloc_table.try_fit(128), Some(0));
        assert_eq!(alloc_table.try_fit(128), Some(128));
        assert!(alloc_table.try_fit(128).is_none());

        assert_eq!(alloc_table.allocs.iter().map(|c| c.length).fold(0, |x, a| x + a), 256);

        alloc_table.free(0);

        assert_eq!(alloc_table.allocs.iter().map(|c| c.length).fold(0, |x, a| x + a), 256);

        assert_eq!(alloc_table.try_fit(7), Some(0));
        assert_eq!(alloc_table.try_fit(7), Some(8));
        assert_eq!(alloc_table.try_fit(5), Some(16));

        assert_eq!(alloc_table.allocs.iter().map(|c| c.length).fold(0, |x, a| x + a), 256);
    }
}
