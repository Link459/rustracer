mod random_integrator;
mod simple_path_integrator;

use std::time::Instant;

use crate::{
    camera::Camera, image::Image, ray::Ray, sampler::Sampler, scene::Scene, vec3::Vec3, Float,
};

trait Integrator {
    fn pixel(&mut self, ray: &Ray, sampler: &dyn Sampler) -> Vec3;
}

struct Renderer<'a> {
    scene: &'a Scene,
    camera: Camera,
    integrator: Box<dyn Integrator>,
    image: Image,
    sampler: Box<dyn Sampler>,
}

impl<'a> Renderer<'a> {
    fn new(
        scene: &'a Scene,
        camera: Camera,
        integrator: impl Integrator + 'static,
        image: Image,
        sampler: impl Sampler + 'static,
    ) -> Self {
        Self {
            scene,
            camera,
            integrator: Box::new(integrator),
            image,
            sampler: Box::new(sampler),
        }
    }

    pub fn render(self) {
        println!(
            "widht: {:?},\nheight: {:?},\nsamples: {:?},\ndepth: {:?}",
            self.scene.camera.config.width,
            self.scene.camera.config.height,
            self.scene.camera.config.samples,
            self.scene.camera.config.max_depth
        );

        println!("starting the render");
        let render_time = Instant::now();
        let mut image = Image::from(&self.scene.camera.config);
        /*image.compute_parallel_present(
            |w, h| {
                return self.trace_ray(w, h, &self.scene.world, &self.scene.lights);
            },
            proxy,
        );*/

        image.compute_parallel(|w, h| {
            Vec3::ZERO
            //return self.trace_ray(w, h, &self.scene.world, &self.scene.lights);
        });

        let time_took = format!("rendering took: {:?}", render_time.elapsed());
        println!("{time_took}");
    }

    /*#[inline(always)]
    pub fn trace_ray(&self, w: u32, h: u32, world: &impl Hittable, lights: &World) -> Vec3 {
        let mut rng = rand::rng();
        let mut color = Vec3::ZERO;

        for s_i in 0..self.sqrt_samples as u64 {
            for s_j in 0..self.sqrt_samples as u64 {
                let u = (w as Float + rng.random_range(0.0..1.0) as Float)
                    / (self.config.width - 1) as Float;
                let v = (h as Float + rng.random_range(0.0..1.0) as Float)
                    / (self.config.height - 1) as Float;
                let r = self.get_ray_stratified(u, v, s_i as Float, s_j as Float);
                //let r = self.get_ray(u, v);
                color += self.ray_color(&r, world, lights, self.config.max_depth);
            }
        }

        return color;
    }

    pub fn ray_color(&self, ray: &Ray, world: &impl Hittable, lights: &World, depth: u32) -> Vec3 {
        return self.integrator.pixel(ray, &self.sampler);
    }*/
}
