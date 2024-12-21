use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    ray::Ray,
    texture::{SolidColor, Texture, TextureStorage},
    vec3::Vec3,
};

use super::{lambertian::random_unit_vector, Material};

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
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let scattered = Ray::new(payload.p, random_unit_vector(), ray.time);
        let attenuation = self.albedo.value(payload.u, payload.v, &payload.p);
        return Some((scattered, attenuation));
    }
}
