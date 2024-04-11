use super::buffer::Buffer;
use super::device::Device;
use ash::vk;

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

pub fn get_buffer_device_address<T: Copy>(device: &Device, buffer: &Buffer<T>) -> u64 {
    unsafe {
        device.get_buffer_device_address(&vk::BufferDeviceAddressInfo {
            buffer: buffer.get_buffer(),
            ..Default::default()
        })
    }
}
