use anyhow::Result;
use ash::vk;

pub struct GpuImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub sampler: Option<vk::Sampler>,
}

pub struct SwapchainImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
}
