use ash::vk;

use crate::gpu::{image::GpuImage, instance, shader};

struct Raytracer {
    main_target: GpuImage,
    pipeline: vk::Pipeline,
}

impl Raytracer {
    pub fn new(instance: &instance::Instance) -> anyhow::Result<Self> {
        let shader =
            shader::Shader::new(instance, "raytrace.spirv", vk::ShaderStageFlags::COMPUTE)?;
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
}
