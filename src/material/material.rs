use serde::{Deserialize, Serialize};

use crate::{hittable::HitPayload, ray::Ray, vec3::Vec3};

use super::{
    dielectric::Dielectric, isotropic::Isotropic, lambertian::Lambertian, metal::Metal,
    DiffuseLight,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MaterialStorage {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
    Isotropic(Isotropic),
}

#[macro_export]
macro_rules! into_mat {
    ($id:ident) => {
        impl Into<crate::material::material::MaterialStorage> for $id {
            fn into(self) -> crate::material::material::MaterialStorage {
                crate::material::material::MaterialStorage::$id(self)
            }
        }
    };
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

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)>;
    fn emitted(&self, _u: f64, _v: f64, _p: &Vec3) -> Vec3 {
        return Vec3::ZERO;
    }
}
