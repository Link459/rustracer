use ash::vk;

use crate::gpu::{descriptor_set::DescriptorHandle, instance, shader};

pub struct Raytracer {
    //main_target: GpuImage,
    pipeline: vk::Pipeline,
}

#[repr(C)]
struct DrawPushConstants {
    image: DescriptorHandle,
}

impl Raytracer {
    pub fn new(instance: &instance::Instance) -> anyhow::Result<Self> {
        let mut shader_path = String::from(env!("SHADER_OUT"));
        shader_path.push_str("raytrace.spirv");
        let shader = shader::Shader::new(instance, &shader_path, vk::ShaderStageFlags::COMPUTE)?;
        let create_info = [vk::ComputePipelineCreateInfo::default()
            .stage(shader.stage_info)
            .layout(instance.pipeline_layout)];

        let pipeline = unsafe {
            instance
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &create_info, None)
                .unwrap()[0]
        };
        shader.destroy(instance);
        return Ok(Self { pipeline });
    }

    pub fn run(
        &self,
        instance: &instance::Instance,
        cmd_buf: vk::CommandBuffer,
        handle: DescriptorHandle,
    ) {
        unsafe {
            instance.cmd_bind_pipeline(cmd_buf, vk::PipelineBindPoint::COMPUTE, self.pipeline);
            let pc = DrawPushConstants { image: handle };
            instance.push_constant(cmd_buf, &pc);
            instance.cmd_dispatch(cmd_buf, 30, 120, 1);
        };
    }

    pub fn destroy(&self, instance: &instance::Instance) {
        unsafe { instance.destroy_pipeline(self.pipeline, None) };
    }
}
