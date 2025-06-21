use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload, material::ScatterPayload, ray::Ray, texture::{Texture, TextureStorage}, vec3::Vec3
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
    fn scatter(&self, _ray: &Ray, _payload: &HitPayload) -> Option<ScatterPayload> {
        return None;
    }

    fn emitted(&self, _ray: &Ray, payload: &HitPayload, u: f64, v: f64, p: &Vec3) -> Vec3 {
        if !payload.front_face {
            return Vec3::ZERO;
        }
        return self.emit.value(u, v, p);
    }
}
