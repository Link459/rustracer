pub mod random_integrator;
pub mod simple_path_integrator;

use rand::Rng;
use std::time::Instant;
use winit::event_loop::EventLoopProxy;

use crate::{
    camera::{random_in_unit_disk, Camera},
    image::Image,
    present::PresentationEvent,
    ray::Ray,
    render::RenderConfig,
    vec3::Vec3,
    Float,
};

pub trait Integrator {
    //fn pixel(&self, ray: &Ray, sampler: &dyn Sampler) -> Vec3;
    fn pixel(&self, ray: &Ray) -> Vec3;
}

pub struct Renderer<I> {
    camera: Camera,
    config: RenderConfig,
    integrator: I,
    pub image: Image,
    proxy: EventLoopProxy<PresentationEvent>,
    //sampler: Box<dyn Sampler>,
}

impl<I> Renderer<I>
where
    I: Integrator + Sync,
{
    pub fn new(
        camera: Camera,
        config: RenderConfig,
        integrator: I,
        proxy: EventLoopProxy<PresentationEvent>,
        //sampler: impl Sampler + 'static,
    ) -> Self {
        let image = Image::new(config.width, config.height, config.samples as f32);

        Self {
            camera,
            config,
            integrator,
            image,
            proxy,
            //sampler: Box::new(sampler),
        }
    }

    pub fn render(&mut self) {
        println!(
            "widht: {:?},\nheight: {:?},\nsamples: {:?},\ndepth: {:?}",
            self.config.width, self.config.height, self.config.samples, self.config.max_depth
        );

        println!("starting the render");
        let render_time = Instant::now();
        let mut image = Image::from(&self.config);

        image.compute_parallel_present(
            |w, h| {
                return self.trace_ray(w, h);
            },
            self.proxy.clone(),
        );

        self.image = image;

        let time_took = format!("rendering took: {:?}", render_time.elapsed());
        println!("{time_took}");
    }

    #[inline(always)]
    pub fn trace_ray(&self, w: u32, h: u32) -> Vec3 {
        let mut rng = rand::rng();
        let mut color = Vec3::ZERO;

        for s_i in 0..self.camera.sqrt_samples as u64 {
            for s_j in 0..self.camera.sqrt_samples as u64 {
                let u = (w as Float + rng.random_range(0.0..1.0) as Float)
                    / (self.config.width - 1) as Float;
                let v = (h as Float + rng.random_range(0.0..1.0) as Float)
                    / (self.config.height - 1) as Float;
                let r = self.get_ray_stratified(u, v, s_i as Float, s_j as Float);
                //let r = self.get_ray(u, v);
                color += self.integrator.pixel(&r);
                //return self.integrator.pixel(ray, &*self.sampler);
            }
        }

        return color / self.config.samples as Float;
    }

    #[inline(always)]
    pub fn get_ray(&self, s: Float, t: Float) -> Ray {
        let origin = if self.camera.lens_radius <= 0.0 {
            self.camera.origin
        } else {
            let rd = self.camera.lens_radius * random_in_unit_disk();
            self.camera.origin + self.camera.cu * rd.x + self.camera.cv * rd.y
        };

        let dir =
            self.camera.lower_left_corner + s * self.camera.horizontal + t * self.camera.vertical
                - self.camera.origin;

        Ray::new(
            origin,
            dir,
            rand::rng().random_range(self.camera.time.min..self.camera.time.max),
        )
    }

    #[inline(always)]
    pub fn get_ray_stratified(&self, s: Float, t: Float, s_i: Float, s_j: Float) -> Ray {
        let offset = self.camera.sample_square_stratified(s_i, s_j);
        let origin = if self.camera.lens_radius <= 0.0 {
            self.camera.origin
        } else {
            self.camera.origin + offset
        };

        let dir =
            self.camera.lower_left_corner + s * self.camera.horizontal + t * self.camera.vertical
                - self.camera.origin;

        Ray::new(
            origin,
            dir,
            rand::rng().random_range(self.camera.time.min..self.camera.time.max),
        )
    }
}
