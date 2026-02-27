use rand::{ RngExt};
use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    material::{same_hemisphere, ScatterPayload},
    pdf::{CosinePDF, PDF},
    ray::Ray,
    texture::{SolidColor, Texture, TextureStorage},
    vec3::Vec3,
    Float,
};

use super::Material;

/// A perfectly diffuse ("matte") material. The apparent brightness remains the same, regardless of
/// the viewing angle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lambertian {
    albedo: TextureStorage,
}

impl Lambertian {
    pub fn new(albedo: impl Into<TextureStorage>) -> Self {
        return Self {
            albedo: albedo.into(),
        };
    }
}

impl From<Vec3> for Lambertian {
    fn from(value: Vec3) -> Self {
        return Self::new(SolidColor::new(value));
    }
}

impl Material for Lambertian {
    fn f(&self, _wi: Vec3, _wo: Vec3) -> Vec3 {
        //BUG: The same hemisphere check does not work
        /*if !same_hemisphere(wi, wo) {
            return Vec3::ZERO;
        }*/
        return self.albedo.value(0.0, 0.0, &Vec3::ZERO) / crate::consts::PI;
    }

    #[inline]
    fn scatter(&self, _wi: &Vec3, payload: &HitPayload) -> Option<ScatterPayload> {
        let albedo = self.albedo.value(payload.u, payload.v, &payload.p) / crate::consts::PI;
        //let d = random_in_unit_disk();
        //let z = (1.0 - (d.x * d.x) - (d.y * d.y)).sqrt().max(0.0);
        //let wo = Vec3::new(d.x, d.y, z);
        let pdf = CosinePDF::new(&payload.normal);
        let wo = pdf.generate();
        let cos_theta = wo.dot(&payload.normal).abs();
        //pdf.value(&wo),
        return Some(ScatterPayload {
            f: albedo,
            wo,
            pdf: cos_theta / crate::consts::PI,
            ..Default::default()
        });
    }

    fn pdf(&self, _wi: &Ray, payload: &HitPayload, wo: &Ray) -> Float {
        let cos_theta = wo.dir.dot(&payload.normal).abs();
        return cos_theta / crate::consts::PI;
        /*let cos_theta = payload.normal.dot(&wo.dir.normalize());
        return if cos_theta < 0.0 {
            0.0
        } else {
            cos_theta / crate::consts::PI
        };*/
        //return 1.0 / (crate::consts::PI * 2.0);
    }
}

pub fn random_unit_vector() -> Vec3 {
    return random_unit_sphere().normalize();
}

pub fn random_unit_sphere() -> Vec3 {
    loop {
        let p = Vec3::random(&mut rand::rng(), -1.0..1.0);
        if p.length_squared() >= 1.0 {
            continue;
        }
        return p;
    }
}

pub fn random_on_hemisphere(normal: &Vec3) -> Vec3 {
    let on_unit_sphere = random_unit_vector();
    if on_unit_sphere.dot(normal) > 0.0 {
        return on_unit_sphere;
    }
    return -on_unit_sphere;
}

pub fn random_cosine_direction() -> Vec3 {
    let mut rng = rand::rng();
    let r1: Float = rng.random_range(0.0..1.0);
    let r2: Float = rng.random_range(0.0..1.0);

    let phi = 2.0 * crate::consts::PI * r1;
    let r2_sqrt = r2.sqrt();
    let x = phi.cos() * r2_sqrt;
    let y = phi.sin() * r2_sqrt;
    let z = (1.0 - r2).sqrt();

    return Vec3::new(x, y, z);
}
