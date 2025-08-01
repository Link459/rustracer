use crate::{
    camera::Camera, hittable::Hittable, integrator::Integrator, interval::Interval,
    material::Material, ray::Ray, render::RenderSettings, vec3::Vec3, world::World, Float,
};

pub struct SimplePathIntegrator<W> {
    camera: Camera,
    world: W,
    lights: World,
    config: RenderSettings,
}

impl<W> SimplePathIntegrator<W>
where
    W: Hittable,
{
    pub fn new(camera: Camera, world: W, lights: World, config: RenderSettings) -> Self {
        Self {
            camera,
            world,
            lights,
            config,
        }
    }

    fn li(&self, mut ray: Ray, mut depth: u32) -> Vec3 {
        /*if depth > self.config.max_depth {
            return Vec3::ZERO;
        }

        let Some((payload, material)) = self.world.hit(&ray, Interval::new(0.001, Float::INFINITY))
        else {
            return self.config.background.call(&ray);
        };

        let emitted = material.emitted(&ray, &payload, payload.u, payload.v, &payload.p);

        let Some(scatter_payload) = material.scatter(&ray.dir, &payload) else {
            return emitted;
        };

        let scattered = Ray::new(payload.p, scatter_payload.wo, ray.time);
        let pdf_value = scatter_payload.pdf;

        let beta = self.li(scattered, depth + 1);

        if pdf_value == 0.0 {
            //return scatter_payload.f * scatter_payload.wo.dot(&payload.normal).abs() * beta;
            return scatter_payload.f * beta;
        } else {
            let f = (scatter_payload.f * scatter_payload.wo.dot(&payload.normal).abs() * beta)
                / pdf_value;

            let color = emitted + f;

            return color;
        }*/

        let mut beta = Vec3::ONE;
        let mut l = Vec3::ZERO;

        let mut specular_bounce = true;

        while beta != Vec3::ZERO {
            let Some((payload, material)) =
                self.world.hit(&ray, Interval::new(0.001, Float::INFINITY))
            else {
                l += beta * self.config.background.call(&ray);
                break;
            };

            if specular_bounce {
                let emitted = material.emitted(&ray.dir, &payload, payload.u, payload.v, &payload.p);

                l += beta * emitted;
            }

            depth += 1;
            if depth > self.config.max_depth {
                break;
            }

            let wi = ray.dir;

            let Some(material_sample) = material.scatter(&wi, &payload) else {
                break;
            };

            ray = Ray::new(payload.p, material_sample.wo, ray.time);
            if material_sample.pdf == 0.0 {
                specular_bounce = true;
                //return material_sample.f * beta;
                beta *= material_sample.f * material_sample.wo.dot(&payload.normal).abs();
            } else {
                beta *= (material_sample.f * material_sample.wo.dot(&payload.normal).abs())
                    / material_sample.pdf;
            }
        }
        return l;
    }
}

impl<W> Integrator for SimplePathIntegrator<W>
where
    W: Hittable,
{
    //fn pixel(&self, ray: &Ray, sampler: &dyn Sampler) -> Vec3 {
    fn pixel(&self, ray: &Ray) -> Vec3 {
        return self.li(*ray, 0);
    }

    fn name() -> &'static str {
        return "SimplePathIntegrator";
    }
}
