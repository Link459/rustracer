use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::{hittable::HitPayload, material::ScatterPayload, ray::Ray, vec3::Vec3, Float};

use super::Material;

/// A Dielectric material, like glass (ior of ~1.5)
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Dielectric {
    ir: Float,
}

impl Dielectric {
    pub fn new(ir: Float) -> Self {
        return Self { ir };
    }

    fn reflectance(cosine: Float, ref_idx: Float) -> Float {
        // Use Schlick's approximation for reflectance
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    #[inline]
    //fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<ScatterPayload> {
    fn scatter(&self, wi: &Vec3, payload: &HitPayload) -> Option<ScatterPayload> {
        let refraction_ratio = if payload.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        //let unit_direction = ray.dir.normalize();
        let unit_direction = -wi.normalize();

        let cos_theta = ((-1.0) * unit_direction).dot(&payload.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::rng();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let will_reflect = rng.random::<Float>() < Self::reflectance(cos_theta, refraction_ratio);

        let direction = if cannot_refract || will_reflect {
            unit_direction.reflect(&payload.normal)
        } else {
            unit_direction.refract(&payload.normal, refraction_ratio)
        };

        //let scattered = Ray::new(payload.p, direction, ray.time);
        let scattered = Ray::new(payload.p, direction, 0.0);

        return Some(ScatterPayload::without_pdf(
            scattered,
            Vec3::new(1.0, 1.0, 1.0),
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        hittable::HitPayload,
        material::{Dielectric, Material},
        vec3::Vec3,
        Float,
    };

    #[test]
    fn dielectric_helmholtz_reciprocity() {
        let material = Dielectric::new(1.5);
        let payload = HitPayload::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), 0.0, 0.0, 0.0);

        let wi = -(Vec3::ZERO - Vec3::new(10.0, 10.0, 10.0)).normalize();
        let Some(sample1) = material.scatter(&wi, &payload) else {
            panic!();
        };
        let wo = sample1.wo;

        let f_1 = material.f(wi, wo);
        let f_2 = material.f(wo, wi);
        assert_eq!(f_1, f_2);
    }

    #[test]
    fn dielectric_positivity() {
        let material = Dielectric::new(1.5);

        for i in 0..1000 {
            let wo = Vec3::from(i as Float / 1000.0);
            let wi = Vec3::from(1.0 - i as Float / 1000.0);

            let f = material.f(wi, wo);
            assert!(f.x >= 0.0);
            assert!(f.y >= 0.0);
            assert!(f.z >= 0.0);
        }
    }

    #[test]
    fn dielectric_conserving() {
        let material = Dielectric::new(1.5);

        let mut sum = Vec3::ZERO;
        for i in 0..1000 {
            let wo = Vec3::from(i as Float / 1000.0);
            let wi = Vec3::from(1.0 - i as Float / 1000.0);

            let f = material.f(wi, wo);

            sum += f * wo.dot(&wi).abs();
        }

        assert!(sum.x >= 1.0);
        assert!(sum.y >= 1.0);
        assert!(sum.z >= 1.0);
    }
}
