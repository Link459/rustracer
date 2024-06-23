use crate::{hittable::HitPayload, ray::Ray, vec3::Vec3};

use super::{
    dielectric::Dielectric, isotropic::Isotropic, lambertian::Lambertian, metal::Metal,
    DiffuseLight,
};

#[macro_export]
macro_rules! into_mat {
    ($id:ident) => {
        impl Into<Material> for $id {
            fn into(self) -> Material {
                Material::$id(self)
            }
        }
    };
}
#[derive(Clone,Debug)]
pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
    Isotropic(Isotropic),
}

impl Material {
    #[inline]
    pub fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        match self {
            Material::Lambertian(ref m) => m.scatter(ray, payload),
            Material::Metal(ref m) => m.scatter(ray, payload),
            Material::Dielectric(ref m) => m.scatter(ray, payload),
            Material::DiffuseLight(ref m) => m.scatter(ray, payload),
            Material::Isotropic(ref m) => m.scatter(ray, payload),
        }
    }

    #[inline]
    pub fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        match self {
            Material::DiffuseLight(ref m) => m.emitted(u, v, p),
            _ => Vec3::ZERO,
        }
    }
}

pub trait Scatter: Send + Sync {
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)>;
    fn emitted(&self, _u: f64, _v: f64, _p: &Vec3) -> Vec3 {
        return Vec3::ZERO;
    }
}
