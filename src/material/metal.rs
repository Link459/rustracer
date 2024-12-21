use serde::{Deserialize, Serialize};

use crate::vec3::Vec3;

use crate::{hittable::HitPayload, ray::Ray};

use super::{lambertian::random_unit_sphere, Material};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Metal {
    albedo: Vec3,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: f64) -> Self {
        return Self { albedo, fuzz };
    }
}

impl Material for Metal {
    #[inline]
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let reflected = ray.dir.normalize().reflect(&payload.normal);
        let scattered = Ray::new(
            payload.p,
            reflected + self.fuzz * random_unit_sphere(),
            ray.time,
        );
        if Vec3::dot(&scattered.dir, &payload.normal) > 0.0 {
            return Some((scattered, self.albedo));
        }

        return None;
    }
}
