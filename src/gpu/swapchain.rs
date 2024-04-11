use anyhow::Result;
use ash::{extensions::khr, extensions::khr::Surface, vk};
use winit::window::Window;

use super::{
    command_pool::CommandBuffer,
    device::Device,
    image::{GpuImage, SwapchainImage},
    util::find_memorytype_index,
};

pub struct Swapchain {
    surface: vk::SurfaceKHR,
    window: Window,
    swapchain: vk::SwapchainKHR,
    swapchain_loader: khr::Swapchain,
    images: Vec<SwapchainImage>,
    depth_image: GpuImage,
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &Device,
        setup_command_buffer: &CommandBuffer,
        surface_loader: &Surface,
        surface: &vk::SurfaceKHR,
        window: Window,
    ) -> Result<Self> {
        let surface_format = unsafe {
            surface_loader.get_physical_device_surface_formats(device.pdevice, *surface)?
        }[0];

        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(device.pdevice, *surface)?
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
            surface_loader.get_physical_device_surface_present_modes(device.pdevice, *surface)?
        };
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_loader = khr::Swapchain::new(&instance, &device);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
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
                let create_view_info = vk::ImageViewCreateInfo::builder()
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
                    device
                        .device
                        .create_image_view(&create_view_info, None)
                        .unwrap()
                }
            })
            .collect();
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(device.pdevice) };
        let depth_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D16_UNORM)
            .extent(surface_resolution.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let depth_image = unsafe { device.create_image(&depth_image_create_info, None)? };
        let depth_image_memory_req = unsafe { device.get_image_memory_requirements(depth_image) };
        let depth_image_memory_index = find_memorytype_index(
            &depth_image_memory_req,
            &device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .expect("Unable to find suitable memory index for depth image.");

        let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(depth_image_memory_req.size)
            .memory_type_index(depth_image_memory_index);

        let depth_image_memory = unsafe {
            device
                .device
                .allocate_memory(&depth_image_allocate_info, None)
                .unwrap()
        };

        unsafe {
            device
                .device
                .bind_image_memory(depth_image, depth_image_memory, 0)
                .expect("Unable to bind depth image memory")
        };

        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        let draw_commands_reuse_fence = unsafe {
            device
                .device
                .create_fence(&fence_create_info, None)
                .expect("Create fence failed.")
        };
        let setup_commands_reuse_fence = unsafe {
            device
                .device
                .create_fence(&fence_create_info, None)
                .expect("Create fence failed.")
        };

        setup_command_buffer.record_and_submit(
            &device,
            vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            setup_commands_reuse_fence,
            device.graphics_queue,
            &[],
            &[],
            &[],
            |device, setup_command_buffer| {
                let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                    .image(depth_image)
                    .dst_access_mask(
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    )
                    .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .layer_count(1)
                            .level_count(1)
                            .build(),
                    );

                unsafe {
                    device.cmd_pipeline_barrier(
                        *setup_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[*layout_transition_barriers],
                    )
                };
                Ok(())
            },
        )?;

        let depth_image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(depth_image)
            .format(depth_image_create_info.format)
            .view_type(vk::ImageViewType::TYPE_2D);

        let depth_image_view = unsafe {
            device
                .device
                .create_image_view(&depth_image_view_info, None)
                .unwrap()
        };

        let depth_image = GpuImage {
            image: depth_image,
            view: depth_image_view,
            memory: depth_image_memory,
            sampler: None,
        };
        let present_images = present_images
            .iter()
            .enumerate()
            .map(|(i, x)| SwapchainImage {
                image: *x,
                view: present_image_views[i],
            })
            .collect();

        Ok(Self {
            surface: *surface,
            window,
            swapchain,
            swapchain_loader,
            images: present_images,
            depth_image,
        })
    }

    pub fn present(&self, present_queue: vk::Queue, image: vk::Image) -> Result<()> {
        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(&[self.swapchain])
            .build();
        unsafe {
            self.swapchain_loader
                .queue_present(present_queue, &present_info)?
        };
        return Ok(());
    }
}
