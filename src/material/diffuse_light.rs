use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    ray::Ray,
    texture::{Texture, TextureStorage},
    vec3::Vec3,
};

use super::Material;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiffuseLight {
    emit: TextureStorage,
}

impl DiffuseLight {
    pub fn new(emit: impl Into<TextureStorage>) -> Self {
        return Self { emit: emit.into() };
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray: &Ray, _payload: &HitPayload) -> Option<(Ray, Vec3)> {
        return None;
    }

    fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        return self.emit.value(u, v, p);
    }
}
