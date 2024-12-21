use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{println, time::Instant};
use winit::event_loop::EventLoopProxy;

use crate::{
    hittable::Hittable, image::Image, interval::Interval, material::Material,
    present::PresentationEvent, ray::Ray, render::RenderConfig, vec3::Vec3,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub lookfrom: Vec3,
    pub lookat: Vec3,
    pub vup: Vec3,
    pub vfov: f64,
    pub aspect_ratio: f64,
    pub aperture: f64,
    pub focus_dist: f64,
    pub time: Interval,
    pub config: RenderConfig,
}

impl Default for CameraConfig {
    fn default() -> Self {
        let lookfrom = Vec3::new(13.0, 2.0, 3.0);
        let lookat = Vec3::new(0.0, 0.0, 0.0);
        let vup = Vec3::new(0.0, 1.0, 0.0);
        let focus_dist = 10.0;
        let aperture = 0.0;

        return Self {
            lookfrom,
            lookat,
            vup,
            vfov: 20.0,
            aspect_ratio: 16.0 / 9.0,
            aperture,
            focus_dist,
            time: Interval::new(0.0, 1.0),
            config: RenderConfig::default(),
        };
    }
}

impl CameraConfig {
    pub fn from_config(config: RenderConfig) -> Self {
        return CameraConfig {
            config,
            ..Default::default()
        };
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    cu: Vec3,
    cv: Vec3,
    lens_radius: f64,
    time: Interval,
    config: RenderConfig,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        return Self::from_camera_config(config);
    }

    pub fn from_camera_config(config: CameraConfig) -> Self {
        // Vertical field-of-view in degrees
        let theta = std::f64::consts::PI / 180.0 * config.vfov;
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = config.aspect_ratio * viewport_height;

        let cw = (config.lookfrom - config.lookat).normalize();
        let cu = config.vup.cross(&cw).normalize();
        let cv = cw.cross(&cu);
        let h = config.focus_dist * viewport_width * cu;
        let v = config.focus_dist * viewport_height * cv;

        let llc = config.lookfrom - h / 2.0 - v / 2.0 - config.focus_dist * cw;

        return Camera {
            origin: config.lookfrom,
            horizontal: h,
            vertical: v,
            lower_left_corner: llc,
            cu,
            cv,
            lens_radius: config.aperture / 2.0,
            time: config.time,
            config: config.config,
        };
    }

    pub fn get_config(&self) -> &RenderConfig {
        &self.config
    }

    pub fn default_with_config(config: RenderConfig) -> Self {
        return Camera {
            config,
            ..Default::default()
        };
    }

    #[inline]
    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk();
        let offset = self.cu * rd.x + self.cv * rd.y;

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
            rand::thread_rng().gen_range(self.time.min..self.time.max),
        )
    }

    pub fn render(
        self,
        world: impl Hittable,
        proxy: EventLoopProxy<PresentationEvent>,
    ) -> Result<Image> {
        println!(
            "widht: {:?},\nheight: {:?},\nsamples: {:?},\ndepth: {:?}",
            self.config.width, self.config.height, self.config.samples, self.config.max_depth
        );

        let sqrt_samples = (self.config.samples as f64).sqrt();
        let _recip_sqrt_samples = 1.0 / sqrt_samples;

        println!("starting the render");
        let render_time = Instant::now();
        let mut image = Image::from(&self.config);
        image.compute_parallel_present(
            |w, h| {
                return self.trace_ray(w, h, &world);
            },
            proxy,
        );

        let time_took = format!("rendering took: {:?}", render_time.elapsed());
        println!("{time_took}");

        Ok(image)
    }

    #[inline(always)]
    pub fn trace_ray(&self, w: u32, h: u32, world: &impl Hittable) -> Vec3 {
        let mut rng = rand::thread_rng();
        let mut color = Vec3::ZERO;
        for _ in 0..self.config.samples {
            let u = (w as f64 + rng.gen_range(0.0..1.0) as f64) / (self.config.width - 1) as f64;
            let v = (h as f64 + rng.gen_range(0.0..1.0) as f64) / (self.config.height - 1) as f64;
            let r = self.get_ray(u, v);
            color += self.ray_color(&r, world, self.config.max_depth);
        }
        return color;
    }

    pub fn ray_color(&self, ray: &Ray, world: &impl Hittable, depth: u32) -> Vec3 {
        //depth <= 0
        if depth == 0 {
            return Vec3::ZERO;
        }

        if let Some((payload, material)) = world.hit(ray, Interval::new(0.001, f64::INFINITY)) {
            let color_from_emit = material.emitted(payload.u, payload.v, &payload.p);
            if let Some((scattered, attenuation)) = material.scatter(ray, &payload) {
                let color_from_scatter = attenuation * self.ray_color(&scattered, world, depth - 1);
                return color_from_emit + color_from_scatter;
            }
            return color_from_emit;
        }

        return self.config.background.call(ray);
    }
}

impl Default for Camera {
    fn default() -> Self {
        let config = CameraConfig::default();

        return Self::new(config);
    }
}

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

#[inline(always)]
fn random_in_unit_disk() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let p = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0);
        if p.length_squared() < 1.0 {
            continue;
        }
        return p;
    }
}
