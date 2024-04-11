use anyhow::Result;
use ash::{
    extensions::khr::{self, RayTracingPipeline},
    vk,
};

use super::config::Config;

pub struct Pipeline {
    pipeline: RayTracingPipeline,
}

impl Pipeline {
    pub fn new(
        pipeline_loader: khr::RayTracingPipeline,
        deferred_op: vk::DeferredOperationKHR,
        cache: Option<vk::PipelineCache>,
        config: Config,
    ) -> Result<Self> {
        let mut pipeline = Self::new_multiple(pipeline_loader, deferred_op, cache, &[config])?;
        return Ok(pipeline.remove(0));
    }
    pub fn new_multiple(
        pipeline_loader: khr::RayTracingPipeline,
        deferred_op: vk::DeferredOperationKHR,
        cache: Option<vk::PipelineCache>,
        configs: &[Config],
    ) -> Result<Vec<Self>> {
        let pipeline_create_infos = configs
            .iter()
            .map(|config| {
                let stages = config
                    .shaders
                    .iter()
                    .map(|x| x.stage_info)
                    .collect::<Vec<_>>();
                let pipline_create_info = vk::RayTracingPipelineCreateInfoKHR::builder()
                    .stages(&stages.as_slice())
                    .groups(config.groups)
                    .max_pipeline_ray_recursion_depth(config.recursion_depth)
                    .layout(config.layout)
                    .build();
                pipline_create_info
            })
            .collect::<Vec<_>>();
        //TODO: create the raytracing pipelines
        let pipelines = match cache {
            Some(ref cache) => unsafe {
                pipeline_loader.create_ray_tracing_pipelines(
                    deferred_op,
                    *cache,
                    pipeline_create_infos.as_slice(),
                    None,
                )?
            },
            None => todo!(),
        };

        todo!()
    }
}
