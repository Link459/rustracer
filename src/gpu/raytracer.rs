use ash::vk;

use crate::gpu::{image::GpuImage, instance, shader};

pub struct Raytracer {
    //main_target: GpuImage,
    pipeline: vk::Pipeline,
}

impl Raytracer {
    pub fn new(instance: &instance::Instance) -> anyhow::Result<Self> {
        let mut shader_path = String::from(env!("SHADER_OUT"));
        shader_path.push_str("raytrace.spirv");
        let shader =
            shader::Shader::new(instance, &shader_path, vk::ShaderStageFlags::COMPUTE)?;
        let create_info = [vk::ComputePipelineCreateInfo::default()
            .stage(shader.stage_info)
            .layout(instance.pipeline_layout)];

        let pipeline = unsafe {
            instance
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), &create_info, None)
                .unwrap()[0]
        };
        return Ok(Self { pipeline });
    }

    pub fn run(&self, cmd_buf: vk::CommandBuffer) {}
}
