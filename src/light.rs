use rand::Rng;

use crate::{hittable::Hittable, model::Model, vec3::Vec3, Float};

pub struct LightSampleContext {
    pub pi: Vec3,
    pub n: Vec3,
    pub ns: Vec3,
}

pub trait Light {
    fn l(&self, p: Vec3, n: Vec3, uv: [Float; 2], w: Vec3) -> Vec3 {
        return Vec3::ZERO;
    }

    fn sample_li(&self, ctx: &LightSampleContext) -> Option<LightSample>;
}

pub struct LightSample {
    pub l: Vec3,
    pub wo: Vec3,
    pub pdf: Float,
}

pub struct UniformLightSampler {
    lights: Vec<Box<dyn Light>>,
}

impl UniformLightSampler {
    pub fn sample<'a>(&'a self) -> Option<&'a dyn Light> {
        if self.lights.is_empty() {
            return None;
        }

        let size = self.lights.len();

        let idx = rand::rng().random_range(0..size);
        return Some(&*self.lights[idx]);
    }
}

struct AreaLight {
    prim: Model,
}

impl Light for AreaLight {
    fn sample_li(&self, ctx: &LightSampleContext) -> Option<LightSample> {
        let wo = self.prim.random(&ctx.n);
        Some(LightSample {
            l: Vec3::ONE,
            wo,
            pdf: self.prim.pdf_value(&ctx.n, &wo),
        })
    }
}
