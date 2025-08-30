use serde::{Deserialize, Serialize};

use crate::material::{same_hemisphere, ScatterPayload};
use crate::vec3::Vec3;

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

impl Material for Metal {
    fn f(&self, _wi: Vec3, _wo: Vec3) -> Vec3 {
        return Vec3::ZERO;
    }

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
    fn equivalence_of_reflectance() {
        let metal = Metal::new(Vec3::ONE, 0.0);
        let payload = HitPayload::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), 0.0, 0.0, 0.0);

        let wi = -(Vec3::ZERO - Vec3::new(10.0, 10.0, 10.0)).normalize();
        if let Some(sample) = metal.scatter(&wi, &payload) {
            let wo = sample.wo;
        }
        panic!("No Sample");
    }
}
