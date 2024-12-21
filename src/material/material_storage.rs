use serde::{Deserialize, Serialize};

use crate::{hittable::HitPayload, ray::Ray, vec3::Vec3};

use super::{
    dielectric::Dielectric, isotropic::Isotropic, lambertian::Lambertian, metal::Metal,
    DiffuseLight, Material,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MaterialStorage {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
    Isotropic(Isotropic),
}

impl Material for MaterialStorage {
    #[inline]
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        match self {
            MaterialStorage::Lambertian(ref m) => m.scatter(ray, payload),
            MaterialStorage::Metal(ref m) => m.scatter(ray, payload),
            MaterialStorage::Dielectric(ref m) => m.scatter(ray, payload),
            MaterialStorage::DiffuseLight(ref m) => m.scatter(ray, payload),
            MaterialStorage::Isotropic(ref m) => m.scatter(ray, payload),
        }
    }

    #[inline]
    fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        match self {
            MaterialStorage::DiffuseLight(ref m) => m.emitted(u, v, p),
            _ => Vec3::ZERO,
        }
    }
}

macro_rules! from_mat {
    ($id:ident) => {
        impl From<crate::material::$id> for MaterialStorage {
            fn from(value: crate::material::$id) -> MaterialStorage {
                return crate::material::MaterialStorage::$id(value);
            }
        }
    };
}

from_mat!(Lambertian);
from_mat!(Metal);
from_mat!(Dielectric);
from_mat!(DiffuseLight);
from_mat!(Isotropic);
