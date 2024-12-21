pub mod dielectric;
pub mod diffuse_light;
pub mod isotropic;
pub mod lambertian;
pub mod material_storage;
pub mod metal;

pub use dielectric::Dielectric;
pub use diffuse_light::DiffuseLight;
pub use isotropic::Isotropic;
pub use lambertian::Lambertian;
pub use metal::Metal;
pub use material_storage::MaterialStorage;

use crate::{hittable::HitPayload, ray::Ray, vec3::Vec3};

pub trait Material: Send + Sync {
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)>;
    fn emitted(&self, _u: f64, _v: f64, _p: &Vec3) -> Vec3 {
        return Vec3::ZERO;
    }
}
