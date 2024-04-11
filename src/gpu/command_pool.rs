use std::ops::Deref;

use super::device::Device;
use anyhow::Result;
use ash::vk;

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
        device: &Device,
        usage: vk::CommandBufferUsageFlags,
        command_buffer_reuse_fence: vk::Fence,
        f: F,
    ) -> Result<()>
    where
        F: FnOnce(&Device, CommandBuffer) -> Result<()>,
    {
        unsafe {
            device.wait_for_fences(&[command_buffer_reuse_fence], true, std::u64::MAX)?;

            device.reset_fences(&[command_buffer_reuse_fence])?;

            device.reset_command_buffer(
                self.buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )?;

            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                flags: usage,
                ..Default::default()
            };

            device.begin_command_buffer(self.buffer, &command_buffer_begin_info)?;
            f(device, self.clone())?;
            device.end_command_buffer(self.buffer)?;
        }
        Ok(())
    }

    pub fn submit(
        &self,
        device: &Device,
        command_buffer_reuse_fence: vk::Fence,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
    ) -> Result<()> {
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&[self.buffer])
            .signal_semaphores(signal_semaphores)
            .build();

        unsafe { device.queue_submit(submit_queue, &[submit_info], command_buffer_reuse_fence)? };
        Ok(())
    }

    pub fn record_and_submit<F>(
        &self,
        device: &Device,
        usage: vk::CommandBufferUsageFlags,
        command_buffer_reuse_fence: vk::Fence,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],

        f: F,
    ) -> Result<()>
    where
        F: FnOnce(&Device, CommandBuffer) -> Result<()>,
    {
        self.record(device, usage, command_buffer_reuse_fence, f)?;
        self.submit(
            device,
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
    pub fn new(device: &Device, queue_index: u32) -> Result<Self> {
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_index);

        let pool = unsafe { device.create_command_pool(&pool_create_info, None) }?;

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(2)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
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
        device: &Device,
        usage: vk::CommandBufferUsageFlags,
        command_buffer_reuse_fence: vk::Fence,
        tasks: Vec<F>,
    ) -> Result<()>
    where
        F: FnOnce(&Device, CommandBuffer) -> Result<()> + Copy,
    {
        for (i, b) in self.command_buffers.iter().enumerate() {
            b.record(device, usage, command_buffer_reuse_fence, tasks[i])?;
        }
        Ok(())
    }

    pub fn submit(
        &self,
        device: &Device,
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
        device: &Device,
        usage: vk::CommandBufferUsageFlags,
        command_buffer_reuse_fence: vk::Fence,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
        tasks: Vec<F>,
    ) -> Result<()>
    where
        F: FnOnce(&Device, CommandBuffer) -> Result<()> + Copy,
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
