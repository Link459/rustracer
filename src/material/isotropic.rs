use crate::{
    hittable::HitPayload,
    into_mat,
    ray::Ray,
    texture::{SolidColor, Texture, TextureValue},
    vec3::Vec3,
};

use super::{
    lambertian::random_unit_vector,
    material::{Material, Scatter},
};

#[derive(Clone, Debug)]
pub struct Isotropic {
    albedo: Texture,
}

into_mat!(Isotropic);

impl Isotropic {
    pub fn new(texture: Texture) -> Material {
        Material::Isotropic(Self { albedo: texture })
    }
}

impl From<Vec3> for Isotropic {
    fn from(value: Vec3) -> Self {
        Self {
            albedo: SolidColor::new(value),
        }
    }
}

impl Scatter for Isotropic {
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let scattered = Ray::new(payload.p, random_unit_vector(), ray.time);
        let attenuation = self.albedo.value(payload.u, payload.v, &payload.p);
        return Some((scattered, attenuation));
    }
}
