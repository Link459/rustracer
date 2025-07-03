use std::f64;

use rand::{thread_rng, Rng};

use crate::{
    hittable::Hittable,
    material::lambertian::{random_cosine_direction, random_unit_vector},
    onb::ONB,
    vec3::Vec3,
};

pub trait PDF {
    ///  pdf value for a given direction
    fn value(&self, dir: &Vec3) -> f64;
    /// a random direction weighted by interal PDF distribution
    fn generate(&self) -> Vec3;
}

pub struct SpherePDF;

impl PDF for SpherePDF {
    fn value(&self, _dir: &Vec3) -> f64 {
        return 1.0 / (4.0 * f64::consts::PI);
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

pub struct HittablePDF<'a, M> {
    model: &'a M,
    origin: Vec3,
}

impl<'a, M> HittablePDF<'a, M> {
    pub fn new(model: &'a M, origin: Vec3) -> Self {
        return Self { model, origin };
    }
}

impl<'a, M: Hittable> PDF for HittablePDF<'a, M> {
    fn value(&self, dir: &Vec3) -> f64 {
        return self.model.pdf_value(&self.origin, dir);
    }

    fn generate(&self) -> Vec3 {
        return self.model.random(&self.origin);
    }
}

pub struct MixturePDF<'a, P1> {
    a: Box<dyn PDF>,
    b: &'a P1,
}

impl<'a, P1> MixturePDF<'a, P1> {
    pub fn new(a: Box<dyn PDF>, b: &'a P1) -> Self {
        Self { a, b }
    }
}

impl<P1: PDF> PDF for MixturePDF<'_, P1> {
    fn value(&self, dir: &Vec3) -> f64 {
        return 0.5 * self.a.value(dir) + 0.5 * self.b.value(dir);
    }

    fn generate(&self) -> Vec3 {
        if thread_rng().gen_range(0.0..1.0) < 0.5 {
            return self.a.generate();
        }
        return self.b.generate();
    }
}
