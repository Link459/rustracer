use serde::{Deserialize, Serialize};

use crate::vec3::Vec3;
use crate::{consts, material::ScatterPayload};

use crate::hittable::HitPayload;
use crate::Float;

use super::{lambertian::random_unit_sphere, Material};

/// A metallic material like, Aluminium. The 'fuzz' controls how 'rough' the surface is, the higher
/// the fuzz the less reflective the material is
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Metal {
    albedo: Vec3,
    fuzz: Float,
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: Float) -> Self {
        return Self { albedo, fuzz };
    }
}

fn ggx(NoH: Float, roughness: Float) -> f32 {
    let a = NoH * roughness;
    let k = roughness / (1.0 - NoH * NoH + a * a);
    return k * k * consts::INV_PI;
}

impl Material for Metal {
    #[inline]
    fn scatter(&self, wi: &Vec3, payload: &HitPayload) -> Option<ScatterPayload> {
        let reflected = (-wi).normalize().reflect(&payload.normal);
        let wo = reflected + self.fuzz * random_unit_sphere();
        let f = self.albedo / wo.dot(&payload.normal).abs();
        //let f = self.albedo / wi.dot(&payload.normal).abs();
        return Some(ScatterPayload {
            f,
            wo,
            pdf: 1.0,
            is_specular: true,
        });
        //}

        //return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        hittable::HitPayload,
        material::{Material, Metal},
        vec3::Vec3,
    };

    #[test]
    fn metal_helmholtz_reciprocity() {
        let metal = Metal::new(Vec3::ONE, 0.0);
        let payload = HitPayload::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), 0.0, 0.0, 0.0);

        let wi = -(Vec3::ZERO - Vec3::new(10.0, 10.0, 10.0)).normalize();
        let Some(sample1) = metal.scatter(&wi, &payload) else {
            panic!();
        };
        let wo = sample1.wo;

        let f_1 = metal.f(wi, wo);
        let f_2 = metal.f(wo, wi);
        assert_eq!(f_1, f_2);
    }
}
