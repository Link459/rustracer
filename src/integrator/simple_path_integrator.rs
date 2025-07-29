use crate::{
    camera::Camera, hittable::Hittable, integrator::Integrator, interval::Interval, material::Material, ray::Ray, render::RenderConfig, sampler::Sampler, scene::Scene, vec3::Vec3, world::World, Float
};

pub struct SimplePathIntegrator<W> {
    camera: Camera,
    world: W,
    lights: World,
    config: RenderConfig,
}

impl<W> SimplePathIntegrator<W>
where
    W: Hittable,
{
    pub fn new(camera: Camera, world: W, lights: World, config: RenderConfig) -> Self {
        Self {
            camera,
            world,
            lights,
            config,
        }
    }

    fn radiance(&self, ray: &Ray, depth: u32) -> Vec3 {
        if depth > self.config.max_depth {
            return Vec3::ZERO;
        }

        let Some((payload, material)) = self.world.hit(ray, Interval::new(0.001, Float::INFINITY))
        else {
            return self.config.background.call(ray);
        };

        let color_from_emit = material.emitted(&ray, &payload, payload.u, payload.v, &payload.p);

        //let Some(scatter_payload) = material.scatter(ray, &payload) else {
        let Some(scatter_payload) = material.scatter(&ray.dir, &payload) else {
            return color_from_emit;
        };

        /*match scatter_payload.pdf_ray {
            RayOrPDF::Ray(ray) => {
                return scatter_payload.attenuation
                    * self.ray_color(&ray, world, lights, depth - 1);
            }
            RayOrPDF::PDF(pdf) => {
                if lights.entities.len() == 0 {
                    return scatter_payload.attenuation;
                }

                let light_pdf = HittablePDF::new(lights, payload.p);
                let mixture_pdf = MixturePDF::new(pdf, &light_pdf);

                let scattered = Ray::new(payload.p, mixture_pdf.generate(), ray.time);
                let pdf_value = mixture_pdf.value(&scattered.dir);

                let scattering_pdf = material.scattering_pdf(ray, &payload, &scattered);

                let sample_color = self.ray_color(&scattered, world, lights, depth - 1);

                let color_from_scatter =
                    (scatter_payload.attenuation * scattering_pdf * sample_color) / pdf_value;

                let color = color_from_emit + color_from_scatter;

                return color;
            }
        }*/

        /*if lights.entities.len() == 0 {
            return scatter_payload.attenuation;
        }*/

        /*let light_pdf = HittablePDF::new(lights, payload.p);
        let mixture_pdf = MixturePDF::new(pdf, &light_pdf);*/

        //let scattered = Ray::new(payload.p, mixture_pdf.generate(), ray.time);
        //let pdf_value = mixture_pdf.value(&scattered.dir);

        let scattered = Ray::new(payload.p, scatter_payload.wo, ray.time);
        let pdf_value = scatter_payload.pdf;

        //self.depth -= 1;
        let value = self.radiance(&scattered, depth + 1);
        //let value = self.pixel(&scattered, sampler);
        if pdf_value == 0.0 {
            return scatter_payload.attenuation * value;
        } else {
            let sample_color = value;
            let scattering_pdf = material.scattering_pdf(ray, &payload, &scattered);
            let color_from_scatter =
                (scatter_payload.attenuation * scattering_pdf * sample_color) / pdf_value;

            let color = color_from_emit + color_from_scatter;

            return color;
        }
    }
}

impl<W> Integrator for SimplePathIntegrator<W>
where
    W: Hittable,
{
    //fn pixel(&self, ray: &Ray, sampler: &dyn Sampler) -> Vec3 {
    fn pixel(&self, ray: &Ray) -> Vec3 {
        return self.radiance(ray, 0);
    }

    fn name() -> &'static str {
        return "SimplePathIntegrator";
    }
}
