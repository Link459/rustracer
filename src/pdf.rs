use std::f64;

use crate::{
    hittable::Hittable,
    material::lambertian::{random_cosine_direction, random_unit_vector},
    model::Model,
    onb::ONB,
    vec3::Vec3,
};

pub trait PDF {
    fn value(&self, dir: &Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

pub struct SpherePDF;

impl PDF for SpherePDF {
    fn value(&self, _dir: &Vec3) -> f64 {
        return 1.0 / (4.0 / f64::consts::PI);
    }

    fn generate(&self) -> Vec3 {
        return random_unit_vector();
    }
}

pub struct CosinePDF {
    uvw: ONB,
}

impl CosinePDF {
    pub fn new(n: &Vec3) -> Self {
        return Self { uvw: ONB::new(n) };
    }
}

impl PDF for CosinePDF {
    fn value(&self, dir: &Vec3) -> f64 {
        let cosine_theta = dir.normalize().dot(self.uvw.w());
        return 0.0_f64.max(cosine_theta / f64::consts::PI);
    }

    fn generate(&self) -> Vec3 {
        return self.uvw.transform(&random_cosine_direction());
    }
}

pub struct HittablePDF {
    model: Model,
    origin: Vec3,
}

impl HittablePDF {
    pub fn new(model: impl Into<Model>, origin: Vec3) -> Self {
        return Self {
            model: model.into(),
            origin,
        };
    }
}

impl PDF for HittablePDF {
    fn value(&self, dir: &Vec3) -> f64 {
        return self.model.pdf_value(&self.origin, dir);
    }

    fn generate(&self) -> Vec3 {
        return self.model.random(&self.origin);
    }
}
