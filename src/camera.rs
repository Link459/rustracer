use anyhow::Result;
use image::EncodableLayout;
use rand::Rng;
use std::{fs::File, io::Write, println, time::Instant};

use crate::{
    hittable::Hittable, image::Image, interval::Interval, ray::Ray, render::RenderConfig,
    vec3::Vec3,
};

#[derive(Clone, Copy)]
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
    pub fn new(
        lookfrom: Vec3,
        lookat: Vec3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
        time: Interval,
        config: RenderConfig,
    ) -> Camera {
        // Vertical field-of-view in degrees
        let theta = std::f64::consts::PI / 180.0 * vfov;
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = aspect_ratio * viewport_height;

        let cw = (lookfrom - lookat).normalize();
        let cu = vup.cross(&cw).normalize();
        let cv = cw.cross(&cu);
        let h = focus_dist * viewport_width * cu;
        let v = focus_dist * viewport_height * cv;

        let llc = lookfrom - h / 2.0 - v / 2.0 - focus_dist * cw;

        Camera {
            origin: lookfrom,
            horizontal: h,
            vertical: v,
            lower_left_corner: llc,
            //cw,
            cu,
            cv,
            lens_radius: aperture / 2.0,
            time,
            config,
        }
    }

    pub fn default_with_config(config: RenderConfig) -> Self {
        let mut cam = Self::default();
        cam.config = config;
        return cam;
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

    pub fn render(self, world: impl Hittable) -> Result<()> {
        println!(
            "widht: {:?},\nheight: {:?},\nsamples: {:?},\ndepth: {:?}",
            self.config.width, self.config.height, self.config.samples, self.config.max_depht
        );

        println!("starting the render");
        let render_time = Instant::now();
        let mut image = Image::from(self.config);
        image.compute_parallel(|w, h| {
            let mut rng = rand::thread_rng();
            let mut color = Vec3::ZERO;
            for _ in 0..self.config.samples {
                let u =
                    (w as f64 + rng.gen_range(0.0..1.0) as f64) / (self.config.width - 1) as f64;
                let v =
                    (h as f64 + rng.gen_range(0.0..1.0) as f64) / (self.config.height - 1) as f64;
                let r = self.get_ray(u, v);
                color += self.ray_color(&r, &world, self.config.max_depht);
            }
            return color;
        });

        let time_took = format!("rendering took: {:?}", render_time.elapsed());
        println!("{time_took}");

        let mut file = File::create("out.ppm")?;
        let ppm = format!(
            "P6\n {:?} {:?}\n255\n",
            self.config.width, self.config.height
        );
        file.write(ppm.as_bytes())?;
        file.write(image.buffer.as_bytes())?;

        Ok(())
    }

    pub fn ray_color(&self, ray: &Ray, world: &impl Hittable, depth: u32) -> Vec3 {
        if depth <= 0 {
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

        let bg = self.config.background;
        return bg(ray);
    }
}

impl Default for Camera {
    fn default() -> Self {
        let lookfrom = Vec3::new(13.0, 2.0, 3.0);
        let lookat = Vec3::new(0.0, 0.0, 0.0);
        let vup = Vec3::new(0.0, 1.0, 0.0);
        let dist_to_focus = 10.0;
        let aperture = 0.0;

        return Self::new(
            lookfrom,
            lookat,
            vup,
            20.0,
            16.0 / 9.0 as f64,
            aperture,
            dist_to_focus,
            Interval::new(0.0, 1.0),
            RenderConfig::default(),
        );
    }
}

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

#[inline]
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
