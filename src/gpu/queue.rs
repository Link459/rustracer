use ash::vk;

pub fn query_queue(
    instance: &ash::Instance,
    pdevice: &vk::PhysicalDevice,
    queue_flags: vk::QueueFlags,
) -> Option<usize> {
    unsafe {
        instance
            .get_physical_device_queue_family_properties(*pdevice)
            .iter()
            .enumerate()
            .find_map(|(index, info)| {
                if info.queue_flags.contains(queue_flags) {
                    return Some(index);
                };
                None
            })
    }
}
