use std::{marker::PhantomData, mem::align_of, ops::Deref};

use anyhow::Result;
use ash::{util::Align, vk};

use super::{device::Device, error::GpuError, util::find_memorytype_index};

/// Generic to prevent missuse by mapping different types to it
pub struct Buffer<T: Copy> {
    buffer: UnsafeBuffer,
    panthom: PhantomData<T>,
}

impl<T: Copy> Buffer<T> {
    pub fn new(
        instance: &ash::Instance,
        device: &Device,
        size: u64,
        usage: vk::BufferUsageFlags,
        flags: vk::BufferCreateFlags,
    ) -> Result<Self> {
        Ok(Self {
            buffer: unsafe { UnsafeBuffer::new(instance, device, size, usage, flags) }?,
            panthom: PhantomData::default(),
        })
    }

    pub fn map(&self, device: &Device, data: &[T]) -> Result<()> {
        unsafe { self.buffer.map(device, data) }
    }

    pub fn size(&self) -> u64 {
        self.buffer.size
    }

    pub fn get_buffer(&self) -> vk::Buffer {
        self.buffer.buffer
    }

    pub fn get_memory(&self) -> vk::DeviceMemory {
        self.buffer.memory
    }
}

#[derive(Copy, Clone, Default)]
pub struct UnsafeBuffer {
    pub size: u64,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl UnsafeBuffer {
    pub unsafe fn new(
        instance: &ash::Instance,
        device: &Device,
        size: u64,
        usage: vk::BufferUsageFlags,
        flags: vk::BufferCreateFlags,
    ) -> Result<Self> {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .flags(flags)
            .build();
        let buffer = unsafe { device.create_buffer(&buffer_create_info, None)? };

        let memory_req = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_props =
            unsafe { instance.get_physical_device_memory_properties(device.pdevice) };

        if let Some(memory_type_index) = find_memorytype_index(
            &memory_req,
            &memory_props,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        ) {
            let alloc_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_req.size)
                .memory_type_index(memory_type_index)
                .build();
            let memory = unsafe { device.allocate_memory(&alloc_info, None)? };
            unsafe { device.bind_buffer_memory(buffer, memory, 0)? };

            return Ok(Self {
                size: memory_req.size,
                buffer,
                memory,
            });
        }
        Err(GpuError::Message("could not get memory props").into())
    }

    pub unsafe fn map<T: Copy>(&self, device: &Device, data: &[T]) -> Result<()> {
        let buffer_ptr =
            unsafe { device.map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())? };

        let mut buffer_slice = unsafe { Align::new(buffer_ptr, align_of::<T>() as u64, self.size) };

        buffer_slice.copy_from_slice(data);

        unsafe { device.unmap_memory(self.memory) };

        Ok(())
    }
}

impl Deref for UnsafeBuffer {
    type Target = vk::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
