use rand::RngExt;

use crate::{
    camera::Camera,
    color::luminance,
    hittable::{Hittable, HittableExt},
    integrator::Integrator,
    interval::Interval,
    light::{LightSampleContext, LightStore, UniformLightSampler},
    material::MaterialStore,
    ray::Ray,
    render::RenderSettings,
    vec3::Vec3,
    Float,
};

pub struct SimplePathIntegrator<'world, W> {
    camera: Camera,
    world: &'world W,
    lights: UniformLightSampler,
    materials: MaterialStore,
    config: RenderSettings,
}

impl<'world, W> SimplePathIntegrator<'world, W>
where
    W: Hittable,
{
    pub fn new(
        camera: Camera,
        world: &'world W,
        lights: LightStore,
        materials: MaterialStore,
        config: RenderSettings,
    ) -> Self {
        Self {
            camera,
            world,
            lights: UniformLightSampler::new(lights),
            materials,
            config,
        }
    }

    fn li(&self, mut ray: Ray, mut depth: u32) -> Vec3 {
        let mut beta = Vec3::ONE;
        let mut l = Vec3::ZERO;

        let mut specular_bounce = true;

        while beta != Vec3::ZERO {
            let Some((payload, material_id)) =
                self.world.hit(&ray, Interval::new(0.001, Float::INFINITY))
            else {
                l += beta * self.config.skybox.call(&ray);
                break;
            };

            let material = self.materials.get(material_id);

            //TODO: NEE
            // - fix unoccluded checking not working properly

            let wi = -ray.dir;
            if let Some(sampled_light) = self.lights.sample() {
                let ctx = LightSampleContext {
                    p: payload.p,
                    n: payload.normal,
                };

                if let Some(sample) = sampled_light.light.sample_li(&ctx) {
                    let wo = sample.wo;
                    let f = material.f(wi, wo) * wo.dot(&ctx.n).abs();

                    //if self.unoccluded(payload.p, sample.p) {
                    if self.world.unoccluded(payload.p, sample.p) {
                        l += (beta * f * sample.l) / (sampled_light.p * sample.pdf);
                    }
                }
            }

            if specular_bounce {
                let emitted =
                    material.emitted(&ray.dir, &payload, payload.u, payload.v, &payload.p);
                l += beta * emitted;
            }

            depth += 1;
            if depth > self.config.max_depth {
                break;
            }

            let Some(material_sample) = material.scatter(&wi, &payload) else {
                break;
            };

            let wo = material_sample.wo;
            ray = Ray::new(payload.p, wo, ray.time);

            beta *= (material_sample.f * wo.dot(&payload.normal).abs()) / material_sample.pdf;
            specular_bounce = material_sample.is_specular;

            // Russian-Roulette
            if depth > 2 {
                let p = luminance(beta);
                if rand::rng().random::<Float>() > p {
                    break;
                }
                beta /= p;
            }
        }
        return l;
    }
}

impl<'world, W> Integrator for SimplePathIntegrator<'world, W>
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
