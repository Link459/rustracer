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

use crate::{hittable::HitPayload, pdf::PDF, ray::Ray, vec3::Vec3, Float};

pub enum RayOrPDF {
    Ray(Ray),
    PDF(Box<dyn PDF>),
}

pub struct ScatterPayload {
    pub attenuation: Vec3,
    //TODO: migrate to the proper way of doing it by having an outgoing direction, it's much more
    //sensible than having a pdf structure
    pub wo: Vec3, // -> outgoing direction
    pub pdf: Float,
    //pub pdf_ray: RayOrPDF,
}

impl ScatterPayload {
    /*pub fn new(scattered: Ray, attenuation: Vec3, pdf: Float) -> Self {
        Self {
            scattered,
            attenuation,
            pdf,
        }
    }*/

    pub fn new(attenuation: Vec3, pdf: impl PDF + 'static) -> Self {
        let wo = pdf.generate();
        Self {
            attenuation,
            wo,
            pdf: pdf.value(&wo), //pdf_ray: RayOrPDF::PDF(Box::new(pdf)),
        }
    }

    pub fn without_pdf(scattered: Ray, attenuation: Vec3) -> Self {
        Self {
            attenuation,
            wo: scattered.dir,
            pdf: 0.0,
            //pdf_ray: RayOrPDF::Ray(scattered),
        }
    }
}

pub trait Material: Send + Sync {
    //fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<ScatterPayload>;
    fn scatter(&self, wi: &Vec3, payload: &HitPayload) -> Option<ScatterPayload>;
    fn emitted(&self, _ray: &Ray, _payload: &HitPayload, _u: Float, _v: Float, _p: &Vec3) -> Vec3 {
        return Vec3::ZERO;
    }

    fn scattering_pdf(&self, _incoming: &Ray, _payload: &HitPayload, _scattered: &Ray) -> Float {
        return 0.0;
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DefaultMaterial;

impl DefaultMaterial {
    pub fn new() -> Self {
        Self {}
    }
}

impl Material for DefaultMaterial {
    fn scatter(&self, _ray: &Vec3, _payload: &HitPayload) -> Option<ScatterPayload> {
        return None;
    }
}
