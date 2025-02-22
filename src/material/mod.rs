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
pub use material_storage::MaterialStorage;
pub use metal::Metal;
use serde::{Deserialize, Serialize};

use crate::{hittable::HitPayload, ray::Ray, vec3::Vec3};

struct Scatter {
    scattered: Ray,
    attenuation: Vec3,
    pdf: f64,
}
pub trait Material: Send + Sync {
    /// Ray: the scattered ray,
    /// Vec3: the color attenuation
    /// f64: the pdf value
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3, f64)>;
    fn emitted(&self, _ray: &Ray, _payload: &HitPayload, _u: f64, _v: f64, _p: &Vec3) -> Vec3 {
        return Vec3::ZERO;
    }

    fn scattering_pdf(&self, _incoming: &Ray, _payload: &HitPayload, _scattered: &Ray) -> f64 {
        return 0.0;
    }
}

#[derive(Debug,Clone, Copy,Default, Serialize, Deserialize)]
pub struct DefaultMaterial;

impl DefaultMaterial {
    pub fn new() -> Self {
        Self {}
    }
}

impl Material for DefaultMaterial {
    fn scatter(&self, _ray: &Ray, _payload: &HitPayload) -> Option<(Ray, Vec3, f64)> {
        return None;
    }
}
