use serde::{Deserialize, Serialize};

use crate::{hittable::HitPayload, material::ScatterPayload, ray::Ray, vec3::Vec3, Float};

use super::{
    dielectric::Dielectric, isotropic::Isotropic, lambertian::Lambertian, metal::Metal,
    DefaultMaterial, DiffuseLight, Material,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MaterialStorage {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
    Isotropic(Isotropic),
    Default(DefaultMaterial),
}

impl Material for MaterialStorage {
    #[inline]
    //fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<ScatterPayload> {
    fn scatter(&self, ray: &Vec3, payload: &HitPayload) -> Option<ScatterPayload> {
        match self {
            MaterialStorage::Lambertian(ref m) => m.scatter(ray, payload),
            MaterialStorage::Metal(ref m) => m.scatter(ray, payload),
            MaterialStorage::Dielectric(ref m) => m.scatter(ray, payload),
            MaterialStorage::DiffuseLight(ref m) => m.scatter(ray, payload),
            MaterialStorage::Isotropic(ref m) => m.scatter(ray, payload),
            MaterialStorage::Default(ref m) => m.scatter(ray, payload),
        }
    }

    #[inline]
    fn emitted(&self, ray: &Ray, payload: &HitPayload, u: Float, v: Float, p: &Vec3) -> Vec3 {
        match self {
            MaterialStorage::DiffuseLight(ref m) => m.emitted(ray, payload, u, v, p),
            _ => Vec3::ZERO,
        }
    }

    #[inline]
    fn scattering_pdf(&self, incoming: &Ray, payload: &HitPayload, scattered: &Ray) -> Float {
        match self {
            MaterialStorage::Lambertian(ref m) => m.scattering_pdf(incoming, payload, scattered),
            MaterialStorage::Metal(ref m) => m.scattering_pdf(incoming, payload, scattered),
            MaterialStorage::Dielectric(ref m) => m.scattering_pdf(incoming, payload, scattered),
            MaterialStorage::DiffuseLight(ref m) => m.scattering_pdf(incoming, payload, scattered),
            MaterialStorage::Isotropic(ref m) => m.scattering_pdf(incoming, payload, scattered),
            MaterialStorage::Default(ref m) => m.scattering_pdf(incoming, payload, scattered),
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

impl From<DefaultMaterial> for MaterialStorage {
    fn from(value: DefaultMaterial) -> MaterialStorage {
        return crate::material::MaterialStorage::Default(value);
    }
}
