pub mod auxiliary_integrator;
pub mod random_integrator;
pub mod simple_path_integrator;

pub use auxiliary_integrator::{AlbedoIntegrator, NormalIntegrator};
pub use simple_path_integrator::SimplePathIntegrator;

use rand::Rng;
use std::time::Instant;
use winit::event_loop::EventLoopProxy;

use crate::{
    camera::{random_in_unit_disk, Camera},
    image::Image,
    present::PresentationEvent,
    ray::Ray,
    render::RenderSettings,
    vec3::Vec3,
    Float,
};

pub trait Integrator {
    //fn pixel(&self, ray: &Ray, sampler: &dyn Sampler) -> Vec3;
    fn pixel(&self, ray: &Ray) -> Vec3;

    fn name() -> &'static str {
        return "Integrator";
    }
}

pub struct ImageIntegrator<I> {
    camera: Camera,
    config: RenderSettings,
    integrator: I,
    pub image: Option<Image>,
    use_samples: bool,
    proxy: EventLoopProxy<PresentationEvent>,
    //sampler: Box<dyn Sampler>,
}

impl<I> ImageIntegrator<I>
where
    I: Integrator + Sync,
{
    pub fn new(
        camera: Camera,
        config: RenderSettings,
        integrator: I,
        use_samples: bool,
        proxy: EventLoopProxy<PresentationEvent>,
        //sampler: impl Sampler + 'static,
    ) -> Self {
        Self {
            camera,
            config,
            integrator,
            image: None,
            use_samples,
            proxy,
        }
    }

    pub fn render(&mut self) {
        println!("starting the render using {}...", I::name());
        let render_time = Instant::now();
        let mut image = Image::from(&self.config);

        if self.use_samples {
            image.compute_parallel_present(
                |w, h| {
                    return self.trace_ray(w, h);
                },
                self.proxy.clone(),
            );
        } else {
            image.compute_parallel_present(
                |w, h| {
                    return self.trace_ray_sampleless(w, h);
                },
                self.proxy.clone(),
            );
        }

        self.image = Some(image);

        let time_took = format!("rendering took: {:?}", render_time.elapsed());
        println!("{time_took}");
    }

    pub fn get_image(self) -> Image {
        if let Some(img) = self.image {
            return img;
        }
        panic!();
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
    pub fn trace_ray_sampleless(&self, w: u32, h: u32) -> Vec3 {
        let mut rng = rand::rng();
        let mut color = Vec3::ZERO;

        let u =
            (w as Float + rng.random_range(0.0..1.0) as Float) / (self.config.width - 1) as Float;
        let v =
            (h as Float + rng.random_range(0.0..1.0) as Float) / (self.config.height - 1) as Float;
        let r = self.get_ray(u, v);
        color += self.integrator.pixel(&r);
        return color;
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
