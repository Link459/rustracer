use super::descriptor_set::DescriptorSet;
use crate::gpu::{
    descriptor_set, instance::Instance, raytracer::Raytracer, swapchain::Swapchain,
    FRAMES_IN_FLIGHT,
};
use ash::{
    prelude::VkResult,
    vk::{self, CommandBufferLevel},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalPosition,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::HasDisplayHandle,
    window::Window,
};

pub struct GpuApp {
    window: Option<Window>,
    width: u32,
    height: u32,
    instance: Instance,
    swapchain: Option<Swapchain>,
    frame_index: usize,
    cmd_pool: vk::CommandPool,
    cmd_bufs: [vk::CommandBuffer; FRAMES_IN_FLIGHT],
    descriptor_set: DescriptorSet,
    raytracer: Raytracer,
}

impl GpuApp {
    pub fn new(width: u32, height: u32, event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
        let mut instance = Instance::new(event_loop.display_handle().unwrap().as_raw())?;
        let cmd_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(instance.queue_family_index);
        let cmd_pool = unsafe { instance.device.create_command_pool(&cmd_pool_info, None)? };
        let cmd_buf_info = vk::CommandBufferAllocateInfo::default()
            .level(CommandBufferLevel::PRIMARY)
            .command_pool(cmd_pool)
            .command_buffer_count(FRAMES_IN_FLIGHT as u32);
        let cmd_buf = unsafe { instance.device.allocate_command_buffers(&cmd_buf_info)? }
            .try_into()
            .unwrap();
        let descriptor_set = DescriptorSet::new(&instance)?;
        let push_constant_range = vk::PushConstantRange::default()
            .size(128)
            .offset(0)
            .stage_flags(vk::ShaderStageFlags::ALL);
        let push_ranges = [push_constant_range];
        let set_layouts = [descriptor_set.layout];
        let layout_create_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(&push_ranges)
            .set_layouts(&set_layouts);
        instance.pipeline_layout = unsafe {
            instance
                .device
                .create_pipeline_layout(&layout_create_info, None)?
        };
        let raytracer = Raytracer::new(&instance)?;

        return Ok(GpuApp {
            window: None,
            width,
            height,
            instance,
            swapchain: None,
            frame_index: 0,
            cmd_bufs: cmd_buf,
            cmd_pool,
            descriptor_set,
            raytracer,
        });
    }

    fn main_loop(&mut self) -> ash::prelude::VkResult<()> {
        let Some(swapchain) = &self.swapchain else {
            return Ok(());
        };

        let fences = [swapchain.fences[self.frame_index]];
        unsafe {
            self.instance
                .device
                .wait_for_fences(&fences, true, 100000)
                .unwrap();
            self.instance.device.reset_fences(&fences)?;
        }

        let (acquired_image, _suboptimal) =
            swapchain.acquire(swapchain.image_acquired_semaphores[self.frame_index])?;

        let cmd_buf = self.cmd_bufs[self.frame_index];

        let begin_info = &vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.instance
                .reset_command_buffer(cmd_buf, vk::CommandBufferResetFlags::empty())?;
            self.instance.begin_command_buffer(cmd_buf, begin_info)?;
        }

        let swapchain_image = swapchain.images[acquired_image as usize].image;
        let swapchain_subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1)
            .base_mip_level(0)
            .base_array_layer(0);
        let barrier = [vk::ImageMemoryBarrier2::default()
            .image(swapchain_image)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::GENERAL)
            .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
            .dst_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
            .subresource_range(swapchain_subresource_range)];
        let dep_info = vk::DependencyInfo::default().image_memory_barriers(&barrier);
        unsafe {
            self.instance.cmd_pipeline_barrier2(cmd_buf, &dep_info);
        }

        let mut clear_value = vk::ClearColorValue::default();
        clear_value.float32 = [0.0, 0.0, 1.0, 0.0];
        let image_range = [swapchain_subresource_range];
        unsafe {
            self.instance.cmd_clear_color_image(
                cmd_buf,
                swapchain_image,
                vk::ImageLayout::GENERAL,
                &clear_value,
                &image_range,
            );
        }

        let handle = self.descriptor_set.bind(
            &self.instance,
            swapchain.images[acquired_image as usize].view,
        );
        unsafe {
            self.instance.cmd_bind_descriptor_sets(
                cmd_buf,
                vk::PipelineBindPoint::COMPUTE,
                self.instance.pipeline_layout,
                0,
                &[self.descriptor_set.set],
                &[],
            );
        }
        self.raytracer.run(&self.instance, cmd_buf, handle);

        let barrier = [vk::ImageMemoryBarrier2::default()
            .image(swapchain_image)
            .old_layout(vk::ImageLayout::GENERAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            //.src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
            .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
            .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags2::empty())
            .subresource_range(swapchain_subresource_range)];
        let dep_info = vk::DependencyInfo::default().image_memory_barriers(&barrier);
        unsafe {
            self.instance.cmd_pipeline_barrier2(cmd_buf, &dep_info);
        }
        unsafe {
            self.instance.end_command_buffer(cmd_buf)?;
        }

        let wait_semaphores = [swapchain.image_acquired_semaphores[self.frame_index]];
        let signal_semaphores = [swapchain.render_complete_semaphores[acquired_image as usize]];
        let wait_dst_stage_mask = [vk::PipelineStageFlags::COMPUTE_SHADER];
        let cmd_bufs = [cmd_buf];
        let submit_info = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .signal_semaphores(&signal_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)
            .command_buffers(&cmd_bufs)];
        unsafe {
            self.instance.queue_submit(
                self.instance.graphics_queue,
                &submit_info,
                swapchain.fences[self.frame_index],
            )?;
        }

        self.frame_index = (self.frame_index + 1) % FRAMES_IN_FLIGHT;

        swapchain
            .present(
                self.instance.graphics_queue,
                acquired_image,
                swapchain.render_complete_semaphores[acquired_image as usize],
            )
            .unwrap();

        return Ok(());
    }

    fn destroy(&mut self) {
        unsafe {
            self.instance.device_wait_idle().unwrap();
        }
        if let Some(swapchain) = &self.swapchain {
            swapchain.destroy(&self.instance);
        }

        self.raytracer.destroy(&self.instance);
        self.descriptor_set.destroy(&self.instance);
        unsafe {
            self.instance.destroy_command_pool(self.cmd_pool, None);
        }
        self.instance.destroy();
    }
}

impl ApplicationHandler for GpuApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height))
                    .with_position(winit::dpi::Position::Logical(LogicalPosition::new(
                        600.0, 600.0,
                    )))
                    .with_title("rustracer")
                    .with_resizable(false),
            )
            .unwrap();

        if let Some(swapchain) = &self.swapchain {
            swapchain.destroy(&self.instance);
        }

        self.swapchain =
            Some(Swapchain::new(&self.instance, &window).expect("Failed to create Swapchain"));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Destroyed => {}
            WindowEvent::CloseRequested => {
                self.destroy();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.main_loop().unwrap();
            }
            _ => (),
        }
    }
}
