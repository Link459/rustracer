use std::{marker::PhantomData, mem::align_of, ops::Deref};

use anyhow::Result;
use ash::{util::Align, vk};

use crate::gpu::instance::Instance;

use super::util::find_memorytype_index;

/// Generic to prevent missuse by mapping different types to it
pub struct Buffer<T: Copy> {
    buffer: UnsafeBuffer,
    panthom: PhantomData<T>,
}

impl<T: Copy> Buffer<T> {
    pub fn new(
        instance: &Instance,
        size: u64,
        usage: vk::BufferUsageFlags,
        flags: vk::BufferCreateFlags,
    ) -> Result<Self> {
        Ok(Self {
            buffer: unsafe { UnsafeBuffer::new(instance, size, usage, flags) }?,
            panthom: PhantomData::default(),
        })
    }

    pub fn map(&self, device: &Instance, data: &[T]) -> Result<()> {
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

    pub fn get_address(&self, instance: &Instance) -> vk::DeviceAddress {
        unsafe {
            return self.buffer.get_address(instance);
        }
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
        instance: &Instance,
        size: u64,
        usage: vk::BufferUsageFlags,
        flags: vk::BufferCreateFlags,
    ) -> Result<Self> {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .flags(flags);
        let buffer = unsafe { instance.device.create_buffer(&buffer_create_info, None)? };

        let memory_req = unsafe { instance.device.get_buffer_memory_requirements(buffer) };
        let memory_props =
            unsafe { instance.get_physical_device_memory_properties(instance.pdevice) };

        if let Some(memory_type_index) = find_memorytype_index(
            &memory_req,
            &memory_props,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        ) {
            let alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(memory_req.size)
                .memory_type_index(memory_type_index);
            let memory = unsafe { instance.device.allocate_memory(&alloc_info, None)? };
            unsafe { instance.device.bind_buffer_memory(buffer, memory, 0)? };

            return Ok(Self {
                size: memory_req.size,
                buffer,
                memory,
            });
        }
        todo!();
    }

    pub unsafe fn map<T: Copy>(&self, instance: &Instance, data: &[T]) -> Result<()> {
        let buffer_ptr = unsafe {
            instance
                .device
                .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())?
        };

        let mut buffer_slice = unsafe { Align::new(buffer_ptr, align_of::<T>() as u64, self.size) };

        buffer_slice.copy_from_slice(data);

        unsafe { instance.device.unmap_memory(self.memory) };

        Ok(())
    }

    pub unsafe fn get_address(&self, instance: &Instance) -> vk::DeviceAddress {
        let info = vk::BufferDeviceAddressInfo::default().buffer(self.buffer);
        return instance.device.get_buffer_device_address(&info);
    }
}

impl Deref for UnsafeBuffer {
    type Target = vk::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
