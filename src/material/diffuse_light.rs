use serde::{Deserialize, Serialize};

use crate::{
    Float, hittable::HitPayload, material::ScatterPayload, texture::{Texture, TextureStorage}, vec3::{Vec3, VectorExtensions}
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
    fn scatter(&self, _wi: &Vec3, _payload: &HitPayload) -> Option<ScatterPayload> {
        return None;
    }

    fn emitted(&self, _wi: &Vec3, payload: &HitPayload, u: Float, v: Float, p: &Vec3) -> Vec3 {
        if !payload.front_face {
            return Vec3::zero();
        }
        return self.emit.value(u, v, p);
    }
}
