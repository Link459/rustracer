use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    into_mat,
    ray::Ray,
    texture::{Texture, TextureValue},
    vec3::Vec3,
};

use super::material::{Material, MaterialStorage};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiffuseLight {
    emit: Texture,
}

into_mat!(DiffuseLight);

impl DiffuseLight {
    pub fn new(emit: Texture) -> MaterialStorage {
        MaterialStorage::DiffuseLight(Self { emit })
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray: &Ray, _payload: &HitPayload) -> Option<(Ray, Vec3)> {
        return None;
    }

    fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        return self.emit.value(u, v, &p);
    }
}
