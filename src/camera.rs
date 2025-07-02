use anyhow::Result;
use core::f64;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    println,
    time::Instant,
};
use winit::event_loop::EventLoopProxy;

use crate::{
    hittable::Hittable,
    image::Image,
    interval::Interval,
    material::{Material, ScatterPayload},
    pdf::{CosinePDF, HittablePDF, MixturePDF, PDF},
    present::PresentationEvent,
    ray::Ray,
    render::RenderConfig,
    vec3::Vec3,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Display for CameraConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "looking from: {}", self.lookfrom)?;
        writeln!(f, "looking at: {}", self.lookat)?;
        writeln!(f, "fov: {}", self.vfov)?;
        writeln!(f, "aperture: {}", self.aperture)?;
        writeln!(f, "focus distance: {}", self.focus_dist)?;
        writeln!(f, "time: {}", self.time)?;
        writeln!(f, "{}", self.config)?;
        return Ok(());
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
    sqrt_samples: f64,
    recip_sqrt_samples: f64,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
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

        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);
        let pixel_delta_u = viewport_u / config.config.width as f64;
        let pixel_delta_v = viewport_v / config.config.height as f64;

        let sqrt_samples = (config.config.samples as f64).sqrt();
        let recip_sqrt_samples = 1.0 / sqrt_samples;

        let lens_radius = config.aperture / 2.0;

        return Camera {
            origin: config.lookfrom,
            horizontal: h,
            vertical: v,
            lower_left_corner: llc,
            cu,
            cv,
            lens_radius,
            time: config.time,
            config: config.config,
            sqrt_samples,
            recip_sqrt_samples,
            pixel_delta_u,
            pixel_delta_v,
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

    #[inline(always)]
    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let origin = if self.lens_radius <= 0.0 {
            self.origin
        } else {
            let rd = self.lens_radius * random_in_unit_disk();
            self.origin + self.cu * rd.x + self.cv * rd.y
        };

        /*let offset = self.sample_square();

        let pixel_sample = self.lower_left_corner
            + ((s + offset.x) * self.pixel_delta_u) * self.horizontal
            + ((t + offset.y) * self.pixel_delta_v) * self.vertical;*/
        let dir = self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin;

        Ray::new(
            origin,
            dir,
            rand::thread_rng().gen_range(self.time.min..self.time.max),
        )
    }

    #[inline(always)]
    pub fn get_ray_stratified(&self, s: f64, t: f64, s_i: f64, s_j: f64) -> Ray {
        let offset = self.sample_square_stratified(s_i, s_j);
        let origin = if self.lens_radius <= 0.0 {
            self.origin
        } else {
            self.origin + offset
        };

        let dir = self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin;

        Ray::new(
            origin,
            dir,
            rand::thread_rng().gen_range(self.time.min..self.time.max),
        )
    }

    pub fn render(
        self,
        world: impl Hittable,
        lights: impl Hittable,
        proxy: EventLoopProxy<PresentationEvent>,
    ) -> Result<Image> {
        println!(
            "widht: {:?},\nheight: {:?},\nsamples: {:?},\ndepth: {:?}",
            self.config.width, self.config.height, self.config.samples, self.config.max_depth
        );

        println!("starting the render");
        let render_time = Instant::now();
        let mut image = Image::from(&self.config);
        image.compute_parallel_present(
            |w, h| {
                return self.trace_ray(w, h, &world, &lights);
            },
            proxy,
        );

        let time_took = format!("rendering took: {:?}", render_time.elapsed());
        println!("{time_took}");

        Ok(image)
    }

    #[inline(always)]
    pub fn trace_ray(&self, w: u32, h: u32, world: &impl Hittable, lights: &impl Hittable) -> Vec3 {
        let mut rng = rand::thread_rng();
        let mut color = Vec3::ZERO;

        for s_i in 0..self.sqrt_samples as u64 {
            for s_j in 0..self.sqrt_samples as u64 {
                let u =
                    (w as f64 + rng.gen_range(0.0..1.0) as f64) / (self.config.width - 1) as f64;
                let v =
                    (h as f64 + rng.gen_range(0.0..1.0) as f64) / (self.config.height - 1) as f64;
                let r = self.get_ray_stratified(u, v, s_i as f64, s_j as f64);
                //let r = self.get_ray(u, v);
                color += self.ray_color(&r, world, lights, self.config.max_depth);
            }
        }

        return color;
    }

    pub fn ray_color(
        &self,
        ray: &Ray,
        world: &impl Hittable,
        lights: &impl Hittable,
        depth: u32,
    ) -> Vec3 {
        if depth == 0 {
            return Vec3::ZERO;
        }

        let Some((payload, material)) = world.hit(ray, Interval::new(0.001, f64::INFINITY)) else {
            return self.config.background.call(ray);
        };

        let color_from_emit = material.emitted(&ray, &payload, payload.u, payload.v, &payload.p);

        let Some(scatter_payload) = material.scatter(ray, &payload) else {
            return color_from_emit;
        };

        let ScatterPayload {
            scattered,
            attenuation,
            pdf: _pdf,
        } = scatter_payload;

        /*let color_from_scatter = attenuation * self.ray_color(&scattered, world, lights, depth - 1);
        return color_from_emit + color_from_scatter;*/

        let surface_pdf = CosinePDF::new(&payload.normal);

        //There's a NaN in here so we must find it
        let light_pdf = HittablePDF::new(lights, payload.p);
        let mixture_pdf = MixturePDF::new(&surface_pdf, &light_pdf);

        let scattered = Ray::new(payload.p, mixture_pdf.generate(), ray.time);
        let mut pdf_value = mixture_pdf.value(&scattered.dir);

        /*let scattered = Ray::new(payload.p, surface_pdf.generate(), ray.time);
        let mut pdf_value = surface_pdf.value(&scattered.dir);*/

        /*let scattered = Ray::new(payload.p, light_pdf.generate(), ray.time);
        let mut pdf_value = light_pdf.value(&scattered.dir);*/

        let scattering_pdf = material.scattering_pdf(ray, &payload, &scattered);

        let sample_color = self.ray_color(&scattered, world, lights, depth - 1);

        if pdf_value == 0.0 {
            pdf_value = f64::EPSILON;
        }

        let color_from_scatter = (attenuation * scattering_pdf * sample_color) / pdf_value;

        let color = color_from_emit + color_from_scatter;

        return color;
    }

    pub fn sample_square(&self) -> Vec3 {
        let mut rng = thread_rng();
        return Vec3::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5), 0.0);
    }

    pub fn sample_square_stratified(&self, s_i: f64, s_j: f64) -> Vec3 {
        let mut rng = thread_rng();
        let px = ((s_i + rng.gen_range(0.0..1.0)) * self.recip_sqrt_samples) - 0.5;
        let py = ((s_j + rng.gen_range(0.0..1.0)) * self.recip_sqrt_samples) - 0.5;
        return Vec3::new(px, py, 0.0);
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
