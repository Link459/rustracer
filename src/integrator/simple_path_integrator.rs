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

        let Some((payload, material)) = self.world.hit(ray, Interval::new(0.001, Float::INFINITY))
        else {
            return self.config.background.call(ray);
        };

        let color_from_emit = material.emitted(&ray, &payload, payload.u, payload.v, &payload.p);

        let Some(scatter_payload) = material.scatter(&ray.dir, &payload) else {
            return color_from_emit;
        };

        let scattered = Ray::new(payload.p, scatter_payload.wo, ray.time);
        let pdf_value = scatter_payload.pdf;

        self.depth -= 1;
        let beta = self.li(&scattered, depth + 1);

        if pdf_value == 0.0 {
            return scatter_payload.attenuation * beta;
        } else {
            let color_from_scatter = (scatter_payload.attenuation
                * scatter_payload.wo.dot(&payload.normal).abs()
                * beta)
                / pdf_value;

            let color = color_from_emit + color_from_scatter;

            return color;
        }*/
        let mut beta = Vec3::ONE;
        let mut l = Vec3::ZERO;

        while beta != Vec3::ZERO {
            let Some((payload, material)) =
                self.world.hit(&ray, Interval::new(0.001, Float::INFINITY))
            else {
                return self.config.background.call(&ray);
            };

            depth += 1;
            if depth > self.config.max_depth {
                break;
            }

            let emitted = material.emitted(&ray, &payload, payload.u, payload.v, &payload.p);

            l += beta * emitted;
            let Some(scatter_payload) = material.scatter(&ray.dir, &payload) else {
                break;
            };

            ray = Ray::new(payload.p, scatter_payload.wo, ray.time);
            let pdf_value = scatter_payload.pdf;

            if pdf_value == 0.0 {
                return scatter_payload.attenuation * beta;
            } else {
                let color_from_scatter = (scatter_payload.attenuation
                    * scatter_payload.wo.dot(&payload.normal).abs())
                    / pdf_value;

                beta *= color_from_scatter;
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
