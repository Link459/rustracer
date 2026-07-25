use anyhow::Result;
use ash::vk;

use crate::gpu::instance;

pub struct GpuImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub sampler: Option<vk::Sampler>,
}

impl GpuImage {
    pub fn new(
        instance: instance::Instance,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) {
        let extent = vk::Extent3D::default().width(width).height(height);
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
        let image = unsafe { instance.create_image(&image_info, None) };
    }
}
