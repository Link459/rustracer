use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    hittable::HitPayload,
    material::ScatterPayload,
    pdf::CosinePDF,
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
    #[inline]
    fn scatter(&self, _ray: &Ray, payload: &HitPayload) -> Option<ScatterPayload> {
        //let mut scatter_direction = payload.normal + random_unit_vector();
        //let mut scatter_direction = random_on_hemisphere(&payload.normal);
        /*let uvw = ONB::new(&payload.normal);
        let scatter_direction = uvw.transform(&random_cosine_direction());

        /*if scatter_direction.near_zero() {
            scatter_direction = payload.normal;
        }*/

        let scattered = Ray::new(payload.p, scatter_direction.normalize(), ray.time);
        let pdf = uvw.w().dot(&scattered.dir) / Float::consts::PI;
        return Some(ScatterPayload::new(
            scattered,
            self.albedo.value(payload.u, payload.v, &payload.p),
            pdf,
        ));*/
        return Some(ScatterPayload::new(
            self.albedo.value(payload.u, payload.v, &payload.p),
            CosinePDF::new(&payload.normal),
        ));
    }

    fn scattering_pdf(&self, _incoming: &Ray, payload: &HitPayload, scattered: &Ray) -> Float {
        //let cos_theta = payload.normal.dot(&scattered.dir.normalize());
        //account for minimal error so that there won't be a divide by 0
        //let error = 1e-5;
        /*if cos_theta < 0.0 {
            return 0.0;
        }
        return cos_theta / Float::consts::PI;
        */
        //return 1.0 / (2.0 * Float::consts::PI);

        let cos_theta = payload.normal.dot(&scattered.dir.normalize());
        return if cos_theta < 0.0 {
            0.0
        } else {
            cos_theta / crate::consts::PI
        };
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
