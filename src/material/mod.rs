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

pub struct ScatterPayload {
    pub f: Vec3,
    pub wo: Vec3, // -> outgoing direction
    pub pdf: Float,
    pub is_specular: bool,
}

impl ScatterPayload {
    pub fn new(attenuation: Vec3, pdf: impl PDF + 'static) -> Self {
        let wo = pdf.generate();
        Self {
            f: attenuation,
            wo,
            pdf: pdf.value(&wo), //pdf_ray: RayOrPDF::PDF(Box::new(pdf)),
            is_specular: false,
        }
    }

    pub fn without_pdf(scattered: Ray, attenuation: Vec3) -> Self {
        Self {
            f: attenuation,
            wo: scattered.dir,
            pdf: 1.0,
            is_specular: false,
        }
    }
}

impl Default for ScatterPayload {
    fn default() -> Self {
        return Self {
            f: Vec3::ZERO,
           	wo: Vec3::ZERO,
            pdf: 0.0,
            is_specular: false,
        };
    }
}

pub trait Material: Send + Sync {
    fn f(&self, _wi: Vec3, _wo: Vec3) -> Vec3 {
        return Vec3::ZERO;
    }

    fn scatter(&self, wi: &Vec3, payload: &HitPayload) -> Option<ScatterPayload>;
    fn emitted(&self, _wi: &Vec3, _payload: &HitPayload, _u: Float, _v: Float, _p: &Vec3) -> Vec3 {
        return Vec3::ZERO;
    }

    fn pdf(&self, _incoming: &Ray, _payload: &HitPayload, _scattered: &Ray) -> Float {
        return 0.0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct MaterialId(pub u32);

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MaterialStore {
    //materials: Vec<Box<dyn Material>>,
    materials: Vec<MaterialStorage>,
    current_id: u32,
}

impl MaterialStore {
    pub fn new() -> Self {
        return Self {
            materials: Vec::new(),
            current_id: 0,
        };
    }

    pub fn add<M>(&mut self, mat: M) -> MaterialId
    where
        //M: Material + 'static,
        M: Into<material_storage::MaterialStorage>,
    {
        //self.materials.push(Box::new(mat));
        self.materials.push(mat.into());
        let id = self.current_id;
        self.current_id += 1;
        return MaterialId(id);
    }

    pub fn get(&self, id: MaterialId) -> &dyn Material {
        //return &*self.materials[id.0 as usize];
        return &self.materials[id.0 as usize];
    }
}

impl std::fmt::Debug for MaterialStore {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
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

fn same_hemisphere(w: Vec3, wp: Vec3) -> bool {
    return w.dot(&wp).is_sign_positive();
    //return w.z * wp.z > 0.0;
}
