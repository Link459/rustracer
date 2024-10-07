use std::f64::consts::{FRAC_PI_2, PI};

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::material::{ MaterialStorage},
    model::model::Model,
    ray::Ray,
    vec3::Vec3,
};

#[derive(Clone, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: MaterialStorage,
    bbox: AABB,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64, material: MaterialStorage) -> Model {
        let rvec = Vec3::from(radius);
        return Model::Sphere(Self {
            center,
            radius,
            material,
            bbox: AABB::from((center - rvec, center + rvec)),
        });
    }

    pub fn get_uv(p: &Vec3) -> (f64, f64) {
        let phi = p.z.atan2(p.x);
        let theta = p.y.asin();
        let u = 1.0 - (phi + PI) / (2.0 * PI);
        let v = (theta + FRAC_PI_2) / PI;
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

        payload.set_face_normal(&ray, outward_normal);

        return Some((payload, self.material.clone()));
    }

    #[inline]
    fn bounding_box(&self) -> &AABB {
        return &self.bbox;
    }
}

unsafe impl Send for Sphere {}
unsafe impl Sync for Sphere {}
