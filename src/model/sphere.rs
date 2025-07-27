use std::f64::{
    self,
    consts::{FRAC_PI_2, PI},
};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::MaterialStorage,
    onb::ONB,
    ray::Ray,
    vec3::Vec3,
    Float,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: Float,
    pub material: MaterialStorage,
}

impl Sphere {
    pub fn new(center: Vec3, radius: Float, material: impl Into<MaterialStorage>) -> Self {
        return Self {
            center,
            radius,
            material: material.into(),
        };
    }

    pub fn get_uv(p: &Vec3) -> (Float, Float) {
        let phi = p.z.atan2(p.x);
        let theta = p.y.asin();
        let u = 1.0 - (phi + crate::consts::PI) / (2.0 * crate::consts::PI);
        let v = (theta + crate::consts::FRAC_PI_2) / crate::consts::PI;
        return (u, v);
    }
}

impl Hittable for Sphere {
    #[inline]
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
        let oc = ray.orig - self.center;
        let a = ray.dir.length().powi(2);
        let half_b = oc.dot(&ray.dir);
        let c = oc.length().powi(2) - self.radius.powi(2);

        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0.0 {
            return None;
        }

        // Find the nearest root that lies in the acceptable range
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < ray_t.min || ray_t.max < root {
            root = (-half_b + sqrtd) / a;
            if root < ray_t.min || ray_t.max < root {
                return None;
            }
        }

        let p = ray.at(root);
        let outward_normal = (p - self.center) / self.radius;
        let normal = (p - self.center) / self.radius;
        let (u, v) = Self::get_uv(&normal);
        let mut payload = HitPayload {
            t: root,
            p,
            u,
            v,
            normal,
            front_face: false,
        };

        payload.set_face_normal(ray, outward_normal);

        return Some((payload, self.material.clone()));
    }

    #[inline]
    fn bounding_box(&self) -> AABB {
        let rvec = Vec3::from(self.radius);
        return AABB::from((self.center - rvec, self.center + rvec));
    }

    fn pdf_value(&self, origin: &Vec3, dir: &Vec3) -> Float {
        let ray = Ray::new(*origin, *dir, 0.0);
        let Some((_payload, _material)) = self.hit(&ray, Interval::new(0.001, Float::INFINITY))
        else {
            return 0.0;
        };

        let distance_sq = (self.center.x - origin).length_squared();
        let cos_theta_max = (1.0 - self.radius * self.radius / distance_sq).sqrt();
        let solid_angle = 2.0 * crate::consts::PI * (1.0 - cos_theta_max);
        return 1.0 / solid_angle;
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let direction = self.center.x - origin;
        let distance_squared = direction.length_squared();
        let uvw = ONB::new(&direction);
        return uvw.transform(&random_to_sphere(self.radius, distance_squared));
    }
}

fn random_to_sphere(radius: Float, distance_square: Float) -> Vec3 {
    let r1 = thread_rng().gen_range(0.0..1.0);
    let r2 = thread_rng().gen_range(0.0..1.0);
    let z = 1.0 + r2 * ((1.0 - radius * radius / distance_square).sqrt() - 1.0);

    let phi = 2.0 * crate::consts::PI * r1;

    let sq = (1.0 - z * z).sqrt();
    let x = phi.cos() * sq;
    let y = phi.sin() * sq;
    return Vec3::new(x, y, z);
}

unsafe impl Send for Sphere {}
unsafe impl Sync for Sphere {}
