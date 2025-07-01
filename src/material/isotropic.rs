use core::f64;

use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    material::ScatterPayload,
    ray::Ray,
    texture::{SolidColor, Texture, TextureStorage},
    vec3::Vec3,
};

use super::{lambertian::random_unit_vector, Material};

/// An Isotropic material used for things like Volumes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Isotropic {
    albedo: TextureStorage,
}

impl Isotropic {
    pub fn new(texture: impl Into<TextureStorage>) -> Self {
        return Self {
            albedo: texture.into(),
        };
    }
}

impl From<Vec3> for Isotropic {
    fn from(value: Vec3) -> Self {
        return Self::new(SolidColor::new(value));
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<ScatterPayload> {
        let scattered = Ray::new(payload.p, random_unit_vector(), ray.time);
        let attenuation = self.albedo.value(payload.u, payload.v, &payload.p);
        let pdf = 1.0 / (4.0 * f64::consts::PI);
        return Some(ScatterPayload::new(scattered, attenuation, pdf));
    }

    fn scattering_pdf(&self, _incoming: &Ray, _payload: &HitPayload, _scattered: &Ray) -> f64 {
        return 1.0 / (4.0 * f64::consts::PI);
    }
}
