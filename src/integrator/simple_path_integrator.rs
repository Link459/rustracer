use rand::Rng;

use crate::{
    camera::Camera,
    hittable::Hittable,
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
        let mut beta = Vec3::ONE;
        let mut l = Vec3::ZERO;

        let mut specular_bounce = true;

        while beta != Vec3::ZERO {
            let Some((payload, material_id)) =
                self.world.hit(&ray, Interval::new(0.001, Float::INFINITY))
            else {
                l += beta * self.config.background.call(&ray);
                break;
            };

            let material = self.materials.get(material_id);

            //TODO: get direct light sampling (nee) to work properly

            let wi = -ray.dir;
            if let Some(sampled_light) = self.lights.sample() {
                let ctx = LightSampleContext {
                    p: payload.p,
                    n: payload.normal,
                };

                if let Some(sample) = sampled_light.light.sample_li(&ctx) {
                    let wo = sample.wo;
                    let f = material.f(wi, wo) * wo.dot(&ctx.n).abs();

                    if self.unnocluded(payload.p, sample.p) {
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
            //specular_bounce = material_sample.is_specular;

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
        let dir = p1 - p0;
        let ray = Ray::new(p0, dir, 0.0);

        /*let dir = p0 - p1;
        let ray = Ray::new(p1, dir, 0.0);*/
        let dist = dir.length();
        let hit = self.world.hit(&ray, Interval::new(0.0, dist - 0.0005));
        //let hit = self.world.hit(&ray, Interval::new(0.0, 1.0 - 0.0005));
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
