use std::ops::Deref;

use anyhow::Result;
use ash::vk;

use crate::gpu::instance::Instance;

#[derive(Clone, Copy, Default)]
pub struct CommandBuffer {
    buffer: vk::CommandBuffer,
    in_use: bool,
}

impl Deref for CommandBuffer {
    type Target = vk::CommandBuffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl CommandBuffer {
    pub fn new(buffer: vk::CommandBuffer) -> Self {
        Self {
            buffer,
            in_use: false,
        }
    }

    pub fn record<F>(
        &self,
        instance: &Instance,
        usage: vk::CommandBufferUsageFlags,
        command_buffer_reuse_fence: vk::Fence,
        f: F,
    ) -> Result<()>
    where
        F: FnOnce(&Instance, CommandBuffer) -> Result<()>,
    {
        unsafe {
            instance
                .device
                .wait_for_fences(&[command_buffer_reuse_fence], true, std::u64::MAX)?;

            instance
                .device
                .reset_fences(&[command_buffer_reuse_fence])?;

            instance.device.reset_command_buffer(
                self.buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )?;

            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                flags: usage,
                ..Default::default()
            };

            instance
                .device
                .begin_command_buffer(self.buffer, &command_buffer_begin_info)?;
            f(instance, self.clone())?;
            instance.device.end_command_buffer(self.buffer)?;
        }
        Ok(())
    }

    pub fn submit(
        &self,
        instance: &Instance,
        command_buffer_reuse_fence: vk::Fence,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
    ) -> Result<()> {
        let cmd_bufs = [self.buffer];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&cmd_bufs)
            .signal_semaphores(signal_semaphores);

        unsafe {
            instance.device.queue_submit(
                submit_queue,
                &[submit_info],
                command_buffer_reuse_fence,
            )?
        };
        Ok(())
    }

    pub fn record_and_submit<F>(
        &self,
        instance: &Instance,
        usage: vk::CommandBufferUsageFlags,
        command_buffer_reuse_fence: vk::Fence,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],

        f: F,
    ) -> Result<()>
    where
        F: FnOnce(&Instance, CommandBuffer) -> Result<()>,
    {
        self.record(instance, usage, command_buffer_reuse_fence, f)?;
        self.submit(
            instance,
            command_buffer_reuse_fence,
            submit_queue,
            wait_mask,
            wait_semaphores,
            signal_semaphores,
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct CommandPool {
    pool: vk::CommandPool,
    command_buffers: Vec<CommandBuffer>,
}

impl CommandPool {
    pub fn new(instance: &Instance, queue_index: u32) -> Result<Self> {
        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_index);

        let pool = unsafe { instance.device.create_command_pool(&pool_create_info, None) }?;

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_buffer_count(2)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            instance.device
                .allocate_command_buffers(&command_buffer_allocate_info)?
                .iter()
                .map(|x| CommandBuffer::new(*x))
                .collect()
        };

        Ok(Self {
            pool,
            command_buffers,
        })
    }

    pub fn get_buffers(&self) -> &Vec<CommandBuffer> {
        &self.command_buffers
    }

    pub fn record<F>(
        &self,
        device: &Instance,
        usage: vk::CommandBufferUsageFlags,
        command_buffer_reuse_fence: vk::Fence,
        tasks: Vec<F>,
    ) -> Result<()>
    where
        F: FnOnce(&Instance, CommandBuffer) -> Result<()> + Copy,
    {
        for (i, b) in self.command_buffers.iter().enumerate() {
            b.record(device, usage, command_buffer_reuse_fence, tasks[i])?;
        }
        Ok(())
    }

    pub fn submit(
        &self,
        device: &Instance,
        command_buffer_reuse_fence: vk::Fence,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
    ) -> Result<()> {
        for b in &self.command_buffers {
            b.submit(
                device,
                command_buffer_reuse_fence,
                submit_queue,
                wait_mask,
                wait_semaphores,
                signal_semaphores,
            )?;
        }
        Ok(())
    }

    pub fn record_and_submit<F>(
        &self,
        device: &Instance,
        usage: vk::CommandBufferUsageFlags,
        command_buffer_reuse_fence: vk::Fence,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
        tasks: Vec<F>,
    ) -> Result<()>
    where
        F: FnOnce(&Instance, CommandBuffer) -> Result<()> + Copy,
    {
        for (i, b) in self.command_buffers.iter().enumerate() {
            b.record_and_submit(
                device,
                usage,
                command_buffer_reuse_fence,
                submit_queue,
                wait_mask,
                wait_semaphores,
                signal_semaphores,
                tasks[i],
            )?;
        }
        Ok(())
    }
}
