use rand::Rng;

use crate::{hittable::HitPayload, ray::Ray, vec3::Vec3};

use super::material::{Material, MaterialStorage};
use crate::into_mat;

#[derive(Clone, Copy, Debug)]
pub struct Dielectric {
    ir: f64,
}

into_mat!(Dielectric);

impl Dielectric {
    pub fn new(ir: f64) -> MaterialStorage {
        return MaterialStorage::Dielectric(Self { ir });
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // Use Schlick's approximation for reflectance
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    #[inline]
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let refraction_ratio = if payload.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = ray.dir.normalize();

        let cos_theta = ((-1.0) * unit_direction).dot(&payload.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let will_reflect = rng.gen::<f64>() < Self::reflectance(cos_theta, refraction_ratio);

        let direction = if cannot_refract || will_reflect {
            unit_direction.reflect(&payload.normal)
        } else {
            unit_direction.refract(&payload.normal, refraction_ratio)
        };

        let scattered = Ray::new(payload.p, direction, ray.time);

        return Some((scattered, Vec3::new(1.0, 1.0, 1.0)));
    }
}
