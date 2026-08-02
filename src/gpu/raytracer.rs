use ash::vk;
use nalgebra_glm::{Mat4x4, Vec2, Vec3};

use crate::{
    gpu::{descriptor_set::DescriptorHandle, instance, shader},
    Float,
};

pub struct Raytracer {
    //main_target: GpuImage,
    pipeline: vk::Pipeline,
    size: vk::Extent2D,
}

#[repr(C)]
struct Camera {
    mx: Vec3,
    my: Vec3,
    mw: Vec3,
    clip_to_world: Mat4x4,
}

#[repr(C)]
struct DrawPushConstants {
    image: DescriptorHandle,
    size: Vec2,
    camera: Camera,
}

impl Raytracer {
    pub fn new(instance: &instance::Instance, size: vk::Extent2D) -> anyhow::Result<Self> {
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
        return Ok(Self { pipeline, size });
    }

    pub fn run(
        &self,
        instance: &instance::Instance,
        cmd_buf: vk::CommandBuffer,
        handle: DescriptorHandle,
    ) {
        let center = nalgebra_glm::vec3(0.0, 1.0, 0.0);
        let up = nalgebra_glm::vec3(0.0, 1.0, 0.0);
        let view = nalgebra_glm::look_at_rh(&nalgebra_glm::zero(), &center, &up);
        let perspective = nalgebra_glm::infinite_perspective_rh_zo(
            self.size.width as Float / self.size.height as Float,
            45.0f32.to_radians(),
            0.1,
        );
        let final_mat = perspective * view;
        let transpose = nalgebra_glm::transpose(&final_mat);
        let camera = Camera {
            mx: nalgebra_glm::row(&transpose, 0).xyz(),
            my: nalgebra_glm::row(&transpose, 1).xyz(),
            mw: nalgebra_glm::row(&transpose, 3).xyz(),
            clip_to_world: nalgebra_glm::inverse(&final_mat),
        };
        let size = Vec2::new(self.size.width as Float, self.size.height as Float);
        println!("h");
        unsafe {
            instance.cmd_bind_pipeline(cmd_buf, vk::PipelineBindPoint::COMPUTE, self.pipeline);
            let pc = DrawPushConstants {
                image: handle,
                size: size,
                camera,
            };
            instance.push_constant(cmd_buf, &pc);
            instance.cmd_dispatch(cmd_buf, self.size.width, self.size.height, 1);
        };
    }

    pub fn destroy(&self, instance: &instance::Instance) {
        unsafe { instance.destroy_pipeline(self.pipeline, None) };
    }
}
