use rand::Rng;

use crate::{
    hittable::{HitPayload, Hittable},
    interval::Interval,
    model::Model,
    ray::Ray,
    vec3::Vec3,
    Float,
};

pub struct LightSampleContext {
    pub p: Vec3,
    pub n: Vec3,
}

pub trait Light {
    fn l(&self, _p: Vec3, _n: Vec3, _uv: [Float; 2], _w: Vec3) -> Vec3 {
        return Vec3::ZERO;
    }

    fn sample_li(&self, ctx: &LightSampleContext) -> Option<LightSample>;

    fn pdf(&self, _ctx: &LightSampleContext) -> Float {
        return 0.0;
    }
}

pub struct LightSample {
    pub l: Vec3,
    pub wo: Vec3,
    pub pdf: Float,
    pub p: Vec3,
    //pub light: HitPayload,
}

pub struct SampledLight<'a> {
    pub light: &'a dyn Light,
    pub p: Float,
}

pub struct UniformLightSampler {
    lights: Vec<Box<dyn Light>>,
}

impl UniformLightSampler {
    pub fn new(lights: Vec<Box<dyn Light>>) -> Self {
        return Self { lights };
    }

    pub fn sample<'a>(&'a self) -> Option<SampledLight<'a>> {
        if self.lights.is_empty() {
            return None;
        }

        let size = self.lights.len();

        let idx = rand::rng().random_range(0..size);
        let light = &*self.lights[idx];
        let p = 1.0 / size as Float;
        return Some(SampledLight { light, p });
    }
}

unsafe impl Sync for UniformLightSampler {}
unsafe impl Send for UniformLightSampler {}

pub struct AreaLight {
    prim: Model,
    origin: Vec3,
}

impl AreaLight {
    pub fn new(prim: impl Into<Model>) -> Self {
        let prim = prim.into();
        let origin = prim.bounding_box().center();
        return Self { prim, origin };
    }
}

impl Light for AreaLight {
    fn sample_li(&self, ctx: &LightSampleContext) -> Option<LightSample> {
        let wo = self.prim.random(&ctx.p);
        let p = self.origin;
        

        return Some(LightSample {
            l: Vec3::ONE,
            wo,
            pdf: self.prim.pdf_value(&ctx.p, &wo),
            p,
            //light: payload,
        });
    }
}

pub struct PointLight {
    scale: Float,
    albedo: Vec3,
    p: Vec3,
}

impl Light for PointLight {
    fn sample_li(&self, ctx: &LightSampleContext) -> Option<LightSample> {
        let wo = (self.p - ctx.p).normalize();
        let li = self.scale * self.albedo / distance_squared(self.p, ctx.p);
        return None;
        //return LightSample { l: li, wo, pdf: 1 };
    }
}

fn distance_squared(a: Vec3, b: Vec3) -> Float {
    return (a - b).length_squared();
}

#[derive(Default)]
pub struct LightStore {
    pub lights: Vec<Box<dyn Light>>,
}

impl LightStore {
    pub fn new() -> Self {
        Self { lights: Vec::new() }
    }

    pub fn add<L>(&mut self, light: L)
    where
        L: Light + 'static,
    {
        self.lights.push(Box::new(light));
    }

    pub fn add_area_light(&mut self, model: impl Into<Model>) {
        self.add(AreaLight::new(model));
    }
}
