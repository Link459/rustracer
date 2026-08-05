use ash::{prelude::VkResult, util::Align, vk};
use std::{marker::PhantomData, mem::align_of, ops::Deref};

use crate::gpu::instance::Instance;

use super::util::find_memorytype_index;

/// Generic to prevent missuse by mapping different types to it
pub struct Buffer<T: Copy> {
    buffer: UnsafeBuffer,
    address: vk::DeviceAddress,
    panthom: PhantomData<T>,
}

impl<T: Copy> Buffer<T> {
    pub fn new(
        instance: &Instance,
        size: u64,
        usage: vk::BufferUsageFlags,
        flags: vk::BufferCreateFlags,
    ) -> VkResult<Self> {
        let buffer = unsafe { UnsafeBuffer::new(instance, size, usage, flags) }?;
        let address = if usage.contains(vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS) {
            unsafe { buffer.get_address(instance) }
        } else {
            vk::DeviceAddress::default()
        };
        Ok(Self {
            buffer,
            address,
            panthom: PhantomData::default(),
        })
    }

    pub fn map(&self, instance: &Instance, data: &[T]) -> VkResult<()> {
        unsafe { self.buffer.map(instance, data) }
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

    pub fn get_address(&self) -> vk::DeviceAddress {
        return self.address;
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
    ) -> VkResult<Self> {
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .flags(flags);
        let buffer = unsafe { instance.device.create_buffer(&buffer_create_info, None)? };

        let memory_req = unsafe { instance.device.get_buffer_memory_requirements(buffer) };
        let memory_props = unsafe {
            instance
                .instance
                .get_physical_device_memory_properties(instance.physical_device)
        };

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

    pub unsafe fn map<T: Copy>(&self, instance: &Instance, data: &[T]) -> VkResult<()> {
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
