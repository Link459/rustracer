use serde::{Deserialize, Serialize};

use crate::material::ScatterPayload;
use crate::vec3::Vec3;

use crate::Float;
use crate::{hittable::HitPayload, ray::Ray};

use super::{lambertian::random_unit_sphere, Material};

/// A metallic material like, Aluminium. The 'fuzz' controls how 'rough' the surface is, the higher
/// the fuzz the less reflective the material is
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Metal {
    albedo: Vec3,
    fuzz: Float,
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: Float) -> Self {
        return Self { albedo, fuzz };
    }
}

impl Material for Metal {
    #[inline]
    //fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<ScatterPayload> {
    fn scatter(&self, wi: &Vec3, payload: &HitPayload) -> Option<ScatterPayload> {
        //let reflected = ray.dir.normalize().reflect(&payload.normal);
        let reflected = wi.normalize().reflect(&payload.normal);
        let scattered = Ray::new(
            payload.p,
            reflected + self.fuzz * random_unit_sphere(),
            0.0,//ray.time,
        );
        if Vec3::dot(&scattered.dir, &payload.normal) > 0.0 {
            //return Some(ScatterPayload::new(scattered, self.albedo, 0.0));
            return Some(ScatterPayload::without_pdf(scattered, self.albedo));
        }

        return None;
    }
}
