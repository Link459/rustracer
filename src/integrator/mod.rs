pub mod accumulating_integrator;
pub mod auxiliary_integrator;
pub mod random_integrator;
pub mod simple_path_integrator;
pub mod present_integrator;

pub use auxiliary_integrator::{AlbedoIntegrator, NormalIntegrator};
pub use simple_path_integrator::SimplePathIntegrator;

use rand::RngExt;
use std::time::Instant;

use crate::{
    camera::{random_in_unit_disk, Camera},
    image::Image,
    ray::Ray,
    render::RenderSettings,
    sampler::Sampler,
    settings::Settings,
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

pub struct ImageIntegrator<I, S> {
    camera: Camera,
    config: RenderSettings,
    log_messages: bool,
    integrator: I,
    pub image: Option<Image>,
    use_samples: bool,
    sampler: S,
}

impl<I, S> ImageIntegrator<I, S>
where
    I: Integrator + Sync,
    S: Sampler + Sync,
{
    pub fn new(
        camera: Camera,
        integrator: I,
        settings: &Settings,
        use_samples: bool,
        sampler: S,
        //sampler: impl Sampler + 'static,
    ) -> Self {
        Self {
            camera,
            config: settings.render_settings.clone(),
            log_messages: settings.log_messages,
            integrator,
            image: None,
            use_samples,
            sampler: sampler,
        }
    }

    pub fn render(&mut self) {
        let Self {
            camera,
            config,
            log_messages,
            integrator,
            image,
            use_samples,
            sampler: _,
        } = self;

        if *log_messages {
            println!("rendering using {}...", I::name());
        }
        let render_time = Instant::now();
        if image.is_none() {
            *image = Some(Image::from(&*config));
        }

        let image = image.as_mut().unwrap();

        {
            if *use_samples {
                image.compute_parallel(|w, h| {
                    return Self::trace_ray(&camera, &config, integrator, w, h);
                });
            } else {
                image.compute_parallel(|w, h| {
                    return Self::trace_ray_sampleless(&camera, &config, integrator, w, h);
                });
            }
        }

        if *log_messages {
            let time_took = format!("rendering took: {:?}", render_time.elapsed());
            println!("{time_took}");
        }
    }

    pub fn get_image(self) -> Image {
        if let Some(img) = self.image {
            return img;
        }
        panic!();
    }

    pub fn get_image_ref(&self) -> &Image {
        if let Some(img) = &self.image {
            return img;
        }
        panic!();
    }

    #[inline(always)]
    fn trace_ray(
        camera: &Camera,
        //sampler: &impl Sampler,
        config: &RenderSettings,
        integrator: &impl Integrator,
        w: u32,
        h: u32,
    ) -> Vec3 {
        let mut rng = rand::rng();
        let mut color = Vec3::ZERO;

        for s_i in 0..camera.sqrt_samples as u64 {
            for s_j in 0..camera.sqrt_samples as u64 {
                //let pixel = sampler.sample_pixel(w, h, 0);
                let u = (w as Float + rng.random_range(0.0..1.0) as Float)
                    / (config.width - 1) as Float;
                let v = (h as Float + rng.random_range(0.0..1.0) as Float)
                    / (config.height - 1) as Float;

                let r = Self::get_ray_stratified(camera, u, v, s_i as Float, s_j as Float);
                //let r = Self::get_ray(camera, u, v);
                color += integrator.pixel(&r);
            }
        }

        return color / config.samples as Float;
    }

    #[inline(always)]
    fn trace_ray_sampleless(
        camera: &Camera,
        config: &RenderSettings,
        integrator: &impl Integrator,
        w: u32,
        h: u32,
    ) -> Vec3 {
        let mut rng = rand::rng();
        let mut color = Vec3::ZERO;

        let u = (w as Float + rng.random_range(0.0..1.0) as Float) / (config.width - 1) as Float;
        let v = (h as Float + rng.random_range(0.0..1.0) as Float) / (config.height - 1) as Float;
        let r = Self::get_ray(camera, u, v);
        color += integrator.pixel(&r);
        return color;
    }

    #[inline(always)]
    fn get_ray(camera: &Camera, s: Float, t: Float) -> Ray {
        let origin = if camera.lens_radius <= 0.0 {
            camera.origin
        } else {
            let rd = camera.lens_radius * random_in_unit_disk();
            camera.origin + camera.cu * rd.x + camera.cv * rd.y
        };

        let dir =
            camera.lower_left_corner + s * camera.horizontal + t * camera.vertical - camera.origin;

        Ray::new(
            origin,
            dir,
            rand::rng().random_range(camera.time.min..camera.time.max),
        )
    }

    #[inline(always)]
    fn get_ray_stratified(camera: &Camera, s: Float, t: Float, s_i: Float, s_j: Float) -> Ray {
        let offset = camera.sample_square_stratified(s_i, s_j);
        let origin = if camera.lens_radius <= 0.0 {
            camera.origin
        } else {
            camera.origin + offset
        };

        let dir =
            camera.lower_left_corner + s * camera.horizontal + t * camera.vertical - camera.origin;

        Ray::new(
            origin,
            dir,
            rand::rng().random_range(camera.time.min..camera.time.max),
        )
    }
}
