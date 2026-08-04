use ash::vk;
use nalgebra_glm::{Mat4x4, Vec2, Vec3};

use crate::{
    gpu::{
        descriptor_set::{DescriptorHandle, DescriptorSet},
        image::GpuImage,
        instance,
    },
    Float,
};

pub struct Raytracer {
    raytrace_pipeline: vk::Pipeline,
    accumulate_pipeline: vk::Pipeline,
    main_target: GpuImage,
    accumulate_target: GpuImage,
    size: vk::Extent2D,
    halton_count: u32,
    frame: u32,
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
    jitter: Vec2,
}

#[repr(C)]
struct AccumulatePushConstants {
    frame: u32,
    src: DescriptorHandle,
    accum: DescriptorHandle,
}

fn halton(mut index: u32, base: u32) -> f32 {
    let mut f = 1.0;
    let mut r = 0.0;
    while index > 0 {
        f = f / base as f32;
        r = r + f * (index % base) as f32;
        index = (index as f32 / base as f32).floor() as u32;
    }
    return r;
}

impl Raytracer {
    pub fn new(
        instance: &instance::Instance,
        descriptor: &mut DescriptorSet,
        size: vk::Extent2D,
    ) -> anyhow::Result<Self> {
        let raytrace_pipeline = instance.create_compute_pipeline("raytrace.spirv");
        let accumulate_pipeline = instance.create_compute_pipeline("accumulate.spirv");
        let mut image = GpuImage::new(
            instance,
            size.width,
            size.height,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        image.handle = descriptor.bind(instance, image.view);
        let mut accumulate_image = GpuImage::new(
            instance,
            size.width,
            size.height,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        accumulate_image.handle = descriptor.bind(instance, accumulate_image.view);
        return Ok(Self {
            raytrace_pipeline,
            accumulate_pipeline,
            size,
            main_target: image,
            accumulate_target: accumulate_image,
            halton_count: 0,
            frame: 0,
        });
    }

    fn jitter(&mut self) -> Vec2 {
        let mut jitter = Vec2::default();
        jitter.x = halton(self.halton_count, 2);
        jitter.y = halton(self.halton_count, 3);
        self.halton_count += 1;
        self.halton_count = self.halton_count % 8;
        jitter.x = ((jitter.x - 0.5) / self.size.width as f32) * 2.0;
        jitter.y = ((jitter.y - 0.5) / self.size.height as f32) * 2.0;
        return jitter;
    }

    pub fn run(&mut self, instance: &instance::Instance, cmd_buf: vk::CommandBuffer) {
        instance.image_barrier(
            cmd_buf,
            vk::ImageMemoryBarrier2::default()
                .image(self.main_target.image)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .dst_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE),
        );
        let center = nalgebra_glm::vec3(0.0, 0.0, 1.0);
        let up = nalgebra_glm::vec3(0.0, 1.0, 0.0);
        let view = nalgebra_glm::look_at_lh(&nalgebra_glm::zero(), &center, &up);
        let perspective = nalgebra_glm::perspective_lh_zo(
            self.size.width as Float / self.size.height as Float,
            45.0f32.to_radians(),
            0.1,
            1000.0,
        );
        let final_mat = perspective * view;
        let transpose = nalgebra_glm::transpose(&final_mat);
        let camera = Camera {
            mx: nalgebra_glm::column(&transpose, 0).xyz(),
            my: nalgebra_glm::column(&transpose, 1).xyz(),
            mw: nalgebra_glm::column(&transpose, 3).xyz(),
            clip_to_world: nalgebra_glm::inverse(&final_mat),
        };

        let jitter = self.jitter();
        //println!("{}", jitter);
        let size = Vec2::new(self.size.width as Float, self.size.height as Float);
        let workgroup_count_x = 1 + ((size.x as u32 - 1) / 16);
        let workgroup_count_y = 1 + ((size.y as u32 - 1) / 16);
        unsafe {
            instance.cmd_bind_pipeline(
                cmd_buf,
                vk::PipelineBindPoint::COMPUTE,
                self.raytrace_pipeline,
            );
            let pc = DrawPushConstants {
                image: self.main_target.handle,
                size: size,
                camera,
                jitter,
            };
            instance.push_constant(cmd_buf, &pc);
            //instance.cmd_dispatch(cmd_buf, self.size.width, self.size.height, 1);
            instance.cmd_dispatch(cmd_buf, workgroup_count_x, workgroup_count_y, 1);
        };
    }

    pub fn accumulate_pass(&self, instance: &instance::Instance, cmd_buf: vk::CommandBuffer) {
        //instance.image_barrier(cmd_buf, vk::ImageMemoryBarrier2::default().image(self.main_target.image).);
        let workgroup_count_x = 1 + ((self.size.width - 1) / 16);
        let workgroup_count_y = 1 + ((self.size.height - 1) / 16);

        let pc = AccumulatePushConstants {
            frame: self.frame,
            src: self.main_target.handle,
            accum: self.accumulate_target.handle,
        };
        unsafe {
            instance.cmd_bind_pipeline(
                cmd_buf,
                vk::PipelineBindPoint::COMPUTE,
                self.accumulate_pipeline,
            );
            instance.push_constant(cmd_buf, &pc);
            //instance.cmd_dispatch(cmd_buf, self.size.width, self.size.height, 1);
            instance.cmd_dispatch(cmd_buf, workgroup_count_x, workgroup_count_y, 1);
        };
    }

    pub fn copy_to_image(
        &mut self,
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
                //self.accumulate_target.image,
                vk::ImageLayout::GENERAL,
                image,
                vk::ImageLayout::GENERAL,
                &region,
                vk::Filter::LINEAR,
            );
        }
        self.frame += 1;
    }

    pub fn destroy(&self, instance: &instance::Instance) {
        unsafe {
            self.main_target.destroy(instance);
            self.accumulate_target.destroy(instance);
            instance.destroy_pipeline(self.raytrace_pipeline, None);
            instance.destroy_pipeline(self.accumulate_pipeline, None);
        };
    }
}
