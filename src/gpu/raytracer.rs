use ash::vk;
use nalgebra_glm::{Mat4x4, Vec2, Vec3};

use crate::{
    gpu::{
        descriptor_set::{DescriptorHandle, DescriptorSet},
        image::GpuImage,
        instance, shader,
    },
    Float,
};

pub struct Raytracer {
    main_target: GpuImage,
    handle: DescriptorHandle,
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
    pub fn new(
        instance: &instance::Instance,
        descriptor: &mut DescriptorSet,
        size: vk::Extent2D,
    ) -> anyhow::Result<Self> {
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

        let image = GpuImage::new(
            instance,
            size.width,
            size.height,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        return Ok(Self {
            pipeline,
            handle: descriptor.bind(instance, image.view),
            size,
            main_target: image,
        });
    }

    pub fn run(&self, instance: &instance::Instance, cmd_buf: vk::CommandBuffer) {
        instance.image_barrier(
            cmd_buf,
            vk::ImageMemoryBarrier2::default()
                .image(self.main_target.image)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .dst_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE),
        );
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
        unsafe {
            instance.cmd_bind_pipeline(cmd_buf, vk::PipelineBindPoint::COMPUTE, self.pipeline);
            let pc = DrawPushConstants {
                image: self.handle,
                size: size,
                camera,
            };
            instance.push_constant(cmd_buf, &pc);
            instance.cmd_dispatch(cmd_buf, self.size.width, self.size.height, 1);
        };
    }

    pub fn copy_to_image(
        &self,
        instance: &instance::Instance,
        cmd_buf: vk::CommandBuffer,
        image: vk::Image,
        dst_size: vk::Extent2D,
    ) {
        let src_offsets = [
            vk::Offset3D::default(),
            vk::Offset3D::default()
                .x(self.size.width as i32)
                .y(self.size.height as i32)
                .z(1),
        ];
        let dst_offsets = [
            vk::Offset3D::default(),
            vk::Offset3D::default()
                .x(dst_size.width as i32)
                .y(dst_size.height as i32)
                .z(1),
        ];

        let subresource = vk::ImageSubresourceLayers::default()
            .mip_level(0)
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(1)
            .base_array_layer(0);
        let region = [vk::ImageBlit::default()
            .src_offsets(src_offsets)
            .src_subresource(subresource)
            .dst_offsets(dst_offsets)
            .dst_subresource(subresource)];
        unsafe {
            instance.cmd_blit_image(
                cmd_buf,
                self.main_target.image,
                vk::ImageLayout::GENERAL,
                image,
                vk::ImageLayout::GENERAL,
                &region,
                vk::Filter::LINEAR,
            );
        }
    }

    pub fn destroy(&self, instance: &instance::Instance) {
        unsafe {
            self.main_target.destroy(instance);
            instance.destroy_pipeline(self.pipeline, None)
        };
    }
}
