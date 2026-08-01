use anyhow::Result;
use ash::{khr, vk};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use crate::gpu::FRAMES_IN_FLIGHT;

use super::{image::GpuImage, util::find_memorytype_index};

pub struct SwapchainImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
}

pub struct Swapchain {
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) surface_loader: khr::surface::Instance,
    //window: Window,
    pub(crate) swapchain: vk::SwapchainKHR,
    pub(crate) swapchain_loader: khr::swapchain::Device,
    pub(crate) images: Vec<SwapchainImage>,

    pub(crate) fences: [vk::Fence; FRAMES_IN_FLIGHT],
    pub(crate) image_acquired_semaphores: [vk::Semaphore; FRAMES_IN_FLIGHT],
    pub(crate) render_complete_semaphores: Vec<vk::Semaphore>,
}

impl Swapchain {
    pub fn new(instance: &super::instance::Instance, window: &Window) -> Result<Self> {
        let surface_loader = khr::surface::Instance::new(&instance.entry, &instance.instance);
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance.instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )
        }?;

        let surface_format = unsafe {
            surface_loader.get_physical_device_surface_formats(instance.physical_device, surface)?
        }[0];

        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(instance.physical_device, surface)?
        };
        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }
        let window_size = window.inner_size();
        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: window_size.width,
                height: window_size.height,
            },
            _ => surface_capabilities.current_extent,
        };
        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_capabilities.current_transform
        };
        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(instance.physical_device, surface)?
        };
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_loader = khr::swapchain::Device::new(&instance.instance, &instance.device);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::STORAGE)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        };

        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let present_image_views: Vec<vk::ImageView> = present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::default()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);
                unsafe {
                    instance
                        .device
                        .create_image_view(&create_view_info, None)
                        .unwrap()
                }
            })
            .collect();

        let present_images = present_images
            .iter()
            .enumerate()
            .map(|(i, x)| SwapchainImage {
                image: *x,
                view: present_image_views[i],
            })
            .collect::<Vec<_>>();

        let mut fences = [vk::Fence::null(); FRAMES_IN_FLIGHT];
        for i in 0..FRAMES_IN_FLIGHT {
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            fences[i] = unsafe { instance.device.create_fence(&fence_info, None)? };
        }

        let sem_info = vk::SemaphoreCreateInfo::default();
        let mut image_acquired_semaphores = [vk::Semaphore::null(); FRAMES_IN_FLIGHT];
        for i in 0..FRAMES_IN_FLIGHT {
            image_acquired_semaphores[i] =
                unsafe { instance.device.create_semaphore(&sem_info, None)? };
        }
        let mut render_complete_semaphores = Vec::<vk::Semaphore>::new();
        render_complete_semaphores.reserve(present_images.len());
        for _ in 0..present_images.len() {
            let sem = unsafe { instance.device.create_semaphore(&sem_info, None)? };
            render_complete_semaphores.push(sem);
        }

        Ok(Self {
            surface,
            surface_loader,
            swapchain,
            swapchain_loader,
            images: present_images,
            fences,
            image_acquired_semaphores,
            render_complete_semaphores,
            //depth_image,
        })
    }

    pub fn acquire(&self, semaphore: vk::Semaphore) -> ash::prelude::VkResult<(u32, bool)> {
        unsafe {
            return self.swapchain_loader.acquire_next_image(
                self.swapchain,
                10000,
                semaphore,
                vk::Fence::null(),
            );
        }
    }

    pub fn present(
        &self,
        present_queue: vk::Queue,
        image_idx: u32,
        wait: vk::Semaphore,
    ) -> Result<()> {
        let swapchains = [self.swapchain];
        let images = [image_idx];
        let wait_semaphores = [wait];
        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphores)
            .image_indices(&images);
        unsafe {
            self.swapchain_loader
                .queue_present(present_queue, &present_info)?
        };
        return Ok(());
    }

    pub fn destroy(&self, instance: &super::instance::Instance) {
        unsafe {
            for fence in self.fences {
                instance.destroy_fence(fence, None);
            }
            for sem in self.image_acquired_semaphores {
                instance.destroy_semaphore(sem, None);
            }
            for sem in &self.render_complete_semaphores {
                instance.destroy_semaphore(*sem, None);
            }

            for img in &self.images {
                instance.device.destroy_image_view(img.view, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
