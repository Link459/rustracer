use ash::{
    prelude::VkResult,
    vk::{self},
};
use nalgebra_glm::{Mat4x4, UVec2, Vec2, Vec3};
use winit::keyboard::Key;

use crate::{
    camera::CameraConfig,
    gpu::{
        buffer::Buffer,
        descriptor_set::{DescriptorHandle, DescriptorSet},
        image::GpuImage,
        instance,
        mesh::{GpuMesh, Mesh},
    },
    Float,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Camera {
    mx: Vec3,
    my: Vec3,
    mw: Vec3,
    clip_to_world: Mat4x4,
    origin: Vec3,
    lower_left_corner: Vec3,
    right: Vec3,
    up: Vec3,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        let viewport_height = 2.0 * (config.vfov.to_radians() / 2.0).tan();
        let viewport_width = config.aspect_ratio * viewport_height;

        let up = config.vup.normalize();
        let origin = config.lookfrom;
        let forward = (origin - config.lookat).normalize();
        let right = config.vup.cross(&forward).normalize() * viewport_width;
        let up = viewport_height * up;

        let llc = origin - 0.5 * right - 0.5 * up + config.focus_dist * forward;

        let view = nalgebra_glm::look_at_lh(&nalgebra_glm::zero(), &origin, &config.vup);
        let perspective = nalgebra_glm::perspective_lh_zo(
            config.aspect_ratio,
            config.vfov.to_radians(),
            0.1,
            100.0,
        );
        let final_mat = perspective * view;
        let transpose = nalgebra_glm::transpose(&final_mat);
        return Self {
            mx: nalgebra_glm::column(&transpose, 0).xyz(),
            my: nalgebra_glm::column(&transpose, 1).xyz(),
            mw: nalgebra_glm::column(&transpose, 3).xyz(),
            clip_to_world: nalgebra_glm::inverse(&final_mat),
            origin,
            lower_left_corner: llc,
            right,
            up,
        };
    }
}

pub struct Raytracer {
    raytrace_pipeline: vk::Pipeline,
    accumulate_pipeline: vk::Pipeline,
    main_target: GpuImage,
    accumulate_target: GpuImage,
    camera_buffer: Buffer<Camera>,
    camera_config: CameraConfig,
    clear_accumulation: bool,
    size: vk::Extent2D,
    halton_index: u32,
    frame: u32,
    mesh: Mesh,
}

#[repr(C)]
struct DrawPushConstants {
    image: DescriptorHandle,
    size: Vec2,
    jitter: Vec2,
    camera: vk::DeviceAddress, //Camera,
    mesh: GpuMesh,
}

#[repr(C)]
struct AccumulatePushConstants {
    src: DescriptorHandle,
    accum: DescriptorHandle,
    size: UVec2,
    frame: u32,
    clear: bool,
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
            vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        /*let cmd_buf = instance.begin_single_time_cmd_buf()?;

        instance.end_single_time_cmd_buf(cmd_buf)?;*/
        accumulate_image.handle = descriptor.bind(instance, accumulate_image.view);
        let camera_buffer = Buffer::new(
            instance,
            std::mem::size_of::<Camera>() as u64,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            vk::BufferCreateFlags::empty(),
        )?;
        instance.name("CameraBuffer", camera_buffer.get_buffer())?;
        let mesh = Mesh::new("assets/triangle.obj", instance)?;

        let mut camera_config = CameraConfig::default();
        camera_config.lookfrom = Vec3::new(0.2, 0.0, 1.0);
        camera_config.vfov = 45.0;
        camera_config.aspect_ratio = size.width as Float / size.height as Float;
        camera_config.focus_dist = 1.0;
        return Ok(Self {
            raytrace_pipeline,
            accumulate_pipeline,
            size,
            camera_buffer,
            main_target: image,
            accumulate_target: accumulate_image,
            clear_accumulation: true,
            camera_config,
            halton_index: 0,
            frame: 0,
            mesh,
        });
    }

    fn jitter(&mut self) -> Vec2 {
        let mut jitter = Vec2::default();
        jitter.x = halton(self.halton_index, 2);
        jitter.y = halton(self.halton_index, 3);
        self.halton_index += 1;
        self.halton_index = self.halton_index % 8;
        jitter.x = 2.0 * (jitter.x - 0.5) / self.size.width as f32;
        jitter.y = 2.0 * (jitter.y - 0.5) / self.size.height as f32;
        return jitter;
    }

    pub fn run(
        &mut self,
        instance: &instance::Instance,
        cmd_buf: vk::CommandBuffer,
    ) -> VkResult<()> {
        instance.image_barrier(
            cmd_buf,
            vk::ImageMemoryBarrier2::default()
                .image(self.main_target.image)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .dst_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE),
        );

        let camera = Camera::new(self.camera_config.clone());
        self.camera_buffer.map(instance, &[camera])?;
        let camera_address = self.camera_buffer.get_address();

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
                jitter,
                camera: camera_address, //camera,
                mesh: self.mesh.to_gpu_mesh(),
            };
            instance.push_constant(cmd_buf, &pc);
            //instance.cmd_dispatch(cmd_buf, self.size.width, self.size.height, 1);
            instance.cmd_dispatch(cmd_buf, workgroup_count_x, workgroup_count_y, 1);
        };
        return Ok(());
    }

    pub fn accumulate_pass(&mut self, instance: &instance::Instance, cmd_buf: vk::CommandBuffer) {
        static mut FIRST_TIME: bool = true;

        let mut old_layout = vk::ImageLayout::GENERAL;
        unsafe {
            if FIRST_TIME {
                old_layout = vk::ImageLayout::UNDEFINED;
                FIRST_TIME = false;
            }
        }
        instance.image_barrier(
            cmd_buf,
            vk::ImageMemoryBarrier2::default()
                .image(self.accumulate_target.image)
                .old_layout(old_layout)
                .new_layout(vk::ImageLayout::GENERAL)
                .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .dst_access_mask(
                    vk::AccessFlags2::SHADER_STORAGE_WRITE | vk::AccessFlags2::SHADER_STORAGE_READ,
                )
                .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .dst_access_mask(
                    vk::AccessFlags2::SHADER_STORAGE_WRITE | vk::AccessFlags2::SHADER_STORAGE_READ,
                ),
        );
        instance.image_barrier(
            cmd_buf,
            vk::ImageMemoryBarrier2::default()
                .image(self.main_target.image)
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::GENERAL)
                .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .dst_access_mask(vk::AccessFlags2::SHADER_STORAGE_READ),
        );
        //instance.image_barrier(cmd_buf, vk::ImageMemoryBarrier2::default().image(self.main_target.image).);
        let workgroup_count_x = 1 + ((self.size.width - 1) / 16);
        let workgroup_count_y = 1 + ((self.size.height - 1) / 16);

        let size = UVec2::new(self.size.width, self.size.height);
        println!("before {}", self.clear_accumulation);
        let pc = AccumulatePushConstants {
            frame: self.frame,
            src: self.main_target.handle,
            accum: self.accumulate_target.handle,
            size,
            clear: self.clear_accumulation,
        };
        self.clear_accumulation = false;
        println!("after {}", self.clear_accumulation);
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
        let img = self.accumulate_target.image;
        //self.main_target.image;
        instance.image_barrier(
            cmd_buf,
            vk::ImageMemoryBarrier2::default()
                .image(img)
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::GENERAL)
                .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
                .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::BLIT)
                .dst_access_mask(vk::AccessFlags2::TRANSFER_READ),
        );
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
                img,
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

    pub fn handle_key_presses(&mut self, event: winit::event::KeyEvent) {
        const MOVE_SPEED: Float = 0.03;
        match event.logical_key.as_ref() {
            Key::Character("w") => {
                self.camera_config.lookfrom.z -= MOVE_SPEED;
                self.clear_accumulation = true;
            }
            Key::Character("s") => {
                self.camera_config.lookfrom.z += MOVE_SPEED;
                self.clear_accumulation = true;
            }
            Key::Character("a") => {
                self.camera_config.lookfrom.x -= MOVE_SPEED;
                self.clear_accumulation = true;
            }
            Key::Character("d") => {
                self.camera_config.lookfrom.x += MOVE_SPEED;
                self.clear_accumulation = true;
            }
            _ => {}
        }
    }
}
