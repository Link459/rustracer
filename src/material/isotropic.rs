use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    into_mat,
    ray::Ray,
    texture::{SolidColor, TextureStorage, Texture},
    vec3::Vec3,
};

use super::{
    lambertian::random_unit_vector,
    material::{Material, MaterialStorage},
};

#[derive(Clone, Debug,Serialize,Deserialize)]
pub struct Isotropic {
    albedo: TextureStorage,
}

into_mat!(Isotropic);

impl Isotropic {
    pub fn new(texture: TextureStorage) -> MaterialStorage {
        MaterialStorage::Isotropic(Self { albedo: texture })
    }
}

impl From<Vec3> for Isotropic {
    fn from(value: Vec3) -> Self {
        Self {
            albedo: SolidColor::new(value),
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let scattered = Ray::new(payload.p, random_unit_vector(), ray.time);
        let attenuation = self.albedo.value(payload.u, payload.v, &payload.p);
        return Some((scattered, attenuation));
    }
}
