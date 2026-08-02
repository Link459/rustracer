use ash::{prelude::VkResult, vk};

use crate::gpu::{instance, util::find_memorytype_index};

pub struct GpuImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
}

impl GpuImage {
    pub fn new(
        instance: &instance::Instance,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        memory_prop_flags: vk::MemoryPropertyFlags,
    ) -> VkResult<GpuImage> {
        let extent = vk::Extent3D::default().width(width).height(height).depth(1);
        let image_info = vk::ImageCreateInfo::default()
            .extent(extent)
            .usage(usage)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .image_type(vk::ImageType::TYPE_2D)
            .samples(vk::SampleCountFlags::TYPE_1)
            .mip_levels(1)
            .array_layers(1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image = unsafe { instance.create_image(&image_info, None)? };

        let memory_req = unsafe { instance.device.get_image_memory_requirements(image) };
        let memory_props = unsafe {
            instance
                .instance
                .get_physical_device_memory_properties(instance.physical_device)
        };

        let memory_type_index =
            find_memorytype_index(&memory_req, &memory_props, memory_prop_flags)
                .ok_or(vk::Result::INCOMPLETE)?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(memory_req.size)
            .memory_type_index(memory_type_index);
        let memory = unsafe { instance.device.allocate_memory(&alloc_info, None)? };
        unsafe { instance.device.bind_image_memory(image, memory, 0)? };

        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1)
            .base_mip_level(0)
            .base_array_layer(0);
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .components(vk::ComponentMapping::default())
            .subresource_range(subresource_range);
        let view = unsafe { instance.create_image_view(&view_info, None)? };

        return Ok(Self {
            image,
            memory,
            view,
        });
    }

    pub fn destroy(&self, instance: &instance::Instance) {
        unsafe {
            instance.destroy_image_view(self.view, None);
            instance.destroy_image(self.image, None);
            instance.free_memory(self.memory, None);
        }
    }
}
