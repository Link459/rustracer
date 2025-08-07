use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    material::{lambertian::random_unit_vector, ScatterPayload},
    pdf::SpherePDF,
    ray::Ray,
    texture::{SolidColor, Texture, TextureStorage},
    vec3::Vec3,
    Float,
};

use super::Material;

/// An Isotropic material used for things like Volumes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Isotropic {
    albedo: TextureStorage,
}

impl Isotropic {
    pub fn new(texture: impl Into<TextureStorage>) -> Self {
        return Self {
            albedo: texture.into(),
        };
    }
}

impl From<Vec3> for Isotropic {
    fn from(value: Vec3) -> Self {
        return Self::new(SolidColor::new(value));
    }
}

impl Material for Isotropic {
    //fn scatter(&self, _ray: &Ray, payload: &HitPayload) -> Option<ScatterPayload> {
    fn scatter(&self, _wi: &Vec3, payload: &HitPayload) -> Option<ScatterPayload> {
        let attenuation = self.albedo.value(payload.u, payload.v, &payload.p);
        let scattered = random_unit_vector();
        let pdf = 1.0 / (4.0 * crate::consts::PI);
        return Some(ScatterPayload {
            f: attenuation,
            wo: scattered,
            pdf,
        });
        //return Some(ScatterPayload::new(attenuation, SpherePDF {}));
    }

    fn pdf(&self, _incoming: &Ray, _payload: &HitPayload, _scattered: &Ray) -> Float {
        return 1.0 / (4.0 * crate::consts::PI);
    }
}
