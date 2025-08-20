use rand::Rng;

use crate::{
    camera::Camera,
    hittable::{HitPayload, Hittable},
    integrator::Integrator,
    interval::Interval,
    light::{LightSampleContext, LightStore, UniformLightSampler},
    material::MaterialStore,
    ray::Ray,
    render::RenderSettings,
    vec3::Vec3,
    Float,
};

pub struct SimplePathIntegrator<W> {
    camera: Camera,
    world: W,
    lights: UniformLightSampler,
    materials: MaterialStore,
    config: RenderSettings,
}

impl<W> SimplePathIntegrator<W>
where
    W: Hittable,
{
    pub fn new(
        camera: Camera,
        world: W,
        lights: LightStore,
        materials: MaterialStore,
        config: RenderSettings,
    ) -> Self {
        Self {
            camera,
            world,
            lights: UniformLightSampler::new(lights.lights),
            materials,
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

        //let mut specular_bounce = true;

        while beta != Vec3::ZERO {
            let Some((payload, material_id)) =
                self.world.hit(&ray, Interval::new(0.001, Float::INFINITY))
            else {
                l += beta * self.config.background.call(&ray);
                break;
            };

            let material = self.materials.get(material_id);

            let wi = -ray.dir;
            if let Some(sampled_light) = self.lights.sample() {
                let ctx = LightSampleContext {
                    p: payload.p,
                    n: payload.normal,
                };

                if let Some(sample) = sampled_light.light.sample_li(&ctx) {
                    let wo = sample.wo;
                    let f = material.f(wi, wo) * wo.dot(&ctx.n).abs();

                    if self.unnocluded(payload.p, Vec3::new(343.0, 554.0, 332.0)) {
                        l += beta * f * sample.l / (sampled_light.p * sample.pdf);
                    }
                }
            }

            //if specular_bounce {
            let emitted = material.emitted(&ray.dir, &payload, payload.u, payload.v, &payload.p);

            l += beta * emitted;
            //}

            depth += 1;
            if depth > self.config.max_depth {
                break;
            }

            let wi = ray.dir;

            let Some(material_sample) = material.scatter(&wi, &payload) else {
                break;
            };

            ray = Ray::new(payload.p, material_sample.wo, ray.time);

            beta *= (material_sample.f * material_sample.wo.dot(&payload.normal).abs())
                / material_sample.pdf;

            // Russian-Roulette
            let p = luminance(beta);

            if rand::rng().random::<Float>() > p {
                break;
            }
            beta /= p;
        }
        return l;
    }

    fn unnocluded(&self, p0: Vec3, p1: Vec3) -> bool {
        let ray = Ray::new_ray_to(p0, p1, 0.0);
        let hit = self.world.hit(&ray, Interval::default());
        return hit.is_none();
    }
}

fn luminance(f: Vec3) -> Float {
    f.dot(&Vec3::new(0.2126, 0.7152, 0.0722))
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
