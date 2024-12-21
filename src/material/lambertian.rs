use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    into_mat,
    ray::Ray,
    texture::{SolidColor, Texture, TextureStorage},
    vec3::Vec3,
};

use super::material::{Material, MaterialStorage};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lambertian {
    albedo: TextureStorage,
}

into_mat!(Lambertian);

impl Lambertian {
    pub fn new(albedo: TextureStorage) -> MaterialStorage {
        return MaterialStorage::Lambertian(Self { albedo });
    }
}

impl From<Vec3> for Lambertian {
    fn from(value: Vec3) -> Self {
        Self {
            albedo: SolidColor::new(value),
        }
    }
}

impl Material for Lambertian {
    #[inline]
    fn scatter(&self, ray: &Ray, payload: &HitPayload) -> Option<(Ray, Vec3)> {
        let mut scatter_direction = payload.normal + random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = payload.normal;
        }

        let scattered = Ray::new(payload.p, scatter_direction, ray.time);
        return Some((
            scattered,
            self.albedo.value(payload.u, payload.v, &payload.p),
        ));
    }
}

pub fn random_unit_vector() -> Vec3 {
    return random_unit_sphere().normalize();
}

pub fn random_unit_sphere() -> Vec3 {
    loop {
        let p = Vec3::random(&mut rand::thread_rng(), -1.0..1.0);
        if p.length_squared() >= 1.0 {
            continue;
        }
        return p;
    }
}
