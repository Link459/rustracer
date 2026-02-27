use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::{
    hittable::{HitSampleContext, Hittable},
    model::Model,
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
    pub l: Vec3,    // light
    pub wo: Vec3,   // direction towards the light
    pub pdf: Float, // pdf for the direction towards to the light
    pub p: Vec3,    // position on the light
}

pub struct SampledLight<'a> {
    pub light: &'a dyn Light,
    pub p: Float, // probability of choosing this light
}

pub struct UniformLightSampler {
    lights: LightStore,
}

impl UniformLightSampler {
    pub fn new(lights: LightStore) -> Self {
        return Self { lights };
    }

    pub fn sample<'a>(&'a self) -> Option<SampledLight<'a>> {
        if self.lights.lights.is_empty() {
            return None;
        }

        let size = self.lights.lights.len();

        let idx = rand::rng().random_range(0..size);
        let light = &self.lights.lights[idx];
        let p = 1.0 / size as Float;
        return Some(SampledLight { light, p });
    }
}

unsafe impl Sync for UniformLightSampler {}
unsafe impl Send for UniformLightSampler {}

#[derive(Serialize, Deserialize, Clone)]
pub struct AreaLight {
    prim: Model,
    emit: Vec3,
}

impl AreaLight {
    pub fn new(prim: impl Into<Model>, emit: Vec3) -> Self {
        let prim = prim.into();
        return Self { prim, emit };
    }
}

impl Light for AreaLight {
    fn sample_li(&self, ctx: &LightSampleContext) -> Option<LightSample> {
        let hit_ctx = HitSampleContext { origin: ctx.p };

        let Some(sample) = self.prim.sample(&hit_ctx) else {
            return None;
        };

        let wo = sample.p - ctx.p;
        //let wo = ctx.p - sample.p;

        //ensures the light itself gets lit properly
        if sample.pdf == 0.0 || wo.length_squared() == 0.0 {
            return None;
        }

        let wo = wo.normalize();

        return Some(LightSample {
            l: self.emit,
            wo,
            pdf: sample.pdf,
            p: sample.p,
        });
    }
}

impl Into<LightStorage> for AreaLight {
    fn into(self) -> LightStorage {
        LightStorage::AreaLight(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PointLight {
    scale: Float,
    albedo: Vec3,
    p: Vec3,
}

impl Light for PointLight {
    fn sample_li(&self, ctx: &LightSampleContext) -> Option<LightSample> {
        let _wo = (self.p - ctx.p).normalize();
        let _li = self.scale * self.albedo / distance_squared(self.p, ctx.p);
        return None;
        //return LightSample { l: li, wo, pdf: 1 };
    }
}

impl Into<LightStorage> for PointLight {
    fn into(self) -> LightStorage {
        LightStorage::PointLight(self)
    }
}

fn distance_squared(a: Vec3, b: Vec3) -> Float {
    return (a - b).length_squared();
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LightStorage {
    AreaLight(AreaLight),
    PointLight(PointLight),
}

impl Light for LightStorage {
    fn l(&self, p: Vec3, n: Vec3, uv: [Float; 2], w: Vec3) -> Vec3 {
        match self {
            LightStorage::AreaLight(area_light) => area_light.l(p, n, uv, w),
            LightStorage::PointLight(point_light) => point_light.l(p, n, uv, w),
        }
    }

    fn sample_li(&self, ctx: &LightSampleContext) -> Option<LightSample> {
        match self {
            LightStorage::AreaLight(area_light) => area_light.sample_li(ctx),
            LightStorage::PointLight(point_light) => point_light.sample_li(ctx),
        }
    }

    fn pdf(&self, ctx: &LightSampleContext) -> Float {
        match self {
            LightStorage::AreaLight(area_light) => area_light.pdf(ctx),
            LightStorage::PointLight(point_light) => point_light.pdf(ctx),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct LightStore {
    //pub lights: Vec<Box<dyn Light>>,
    pub lights: Vec<LightStorage>,
}

impl LightStore {
    pub fn new() -> Self {
        Self { lights: Vec::new() }
    }

    pub fn add<L>(&mut self, light: L)
    where
        //L: Light + 'static,
        L: Into<LightStorage> + 'static,
    {
        self.lights.push(light.into());
    }

    pub fn add_area_light(&mut self, model: impl Into<Model>, emit: Vec3) {
        self.add(AreaLight::new(model, emit));
    }

    pub fn len(&self) -> usize {
        return self.lights.len();
    }

    pub fn is_empty(&self) -> bool {
        return self.lights.is_empty();
    }
}
