use serde::{Deserialize, Serialize};

use crate::material::ScatterPayload;
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
    #[inline]
    //fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<ScatterPayload> {
    fn scatter(&self, wi: &Vec3, payload: &HitPayload) -> Option<ScatterPayload> {
        let reflected = wi.normalize().reflect(&payload.normal);
        let scattered = reflected + self.fuzz * random_unit_sphere();
        if Vec3::dot(&scattered, &payload.normal) > 0.0 {
            return Some(ScatterPayload {
                f: self.albedo,
                wo: scattered,
                pdf: 0.0,
                //pdf: 1.0,
            });
        }

        return None;
    }
}
