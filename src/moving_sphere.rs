use std::cell::RefCell;

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::material::Material,
    model::sphere::Sphere,
    ray::Ray,
    vec3::Vec3,
};

#[derive(Clone,Debug)]
pub struct MovingSphere {
    pub center0: Vec3,
    pub center1: Vec3,
    pub time0: f64,
    pub time1: f64,
    pub radius: f64,
    pub material: Material,
    bounding_box: AABB,
}

impl MovingSphere {
    pub fn new(
        center0: Vec3,
        center1: Vec3,
        time0: f64,
        time1: f64,
        radius: f64,
        material: Material,
    ) -> Self {
        return Self {
            center0,
            center1,
            time0,
            time1,
            radius,
            material,
            bounding_box: AABB::default(),
        };
    }

    #[inline]
    pub fn center(&self, time: f64) -> Vec3 {
        return self.center0
            + ((time - self.time0) / (self.time1 - self.time0)) * (self.center1 - self.center0);
    }

    fn mutate_bbox(&self, bbox: RefCell<AABB>) {
        let rvec = Vec3::new(self.radius, self.radius, self.radius);
        let box1 = AABB::from((self.center0 - rvec, self.center0 + rvec));
        let box2 = AABB::from((self.center1 - rvec, self.center1 + rvec));
        bbox.replace(AABB::from((box1, box2)));
    }
}

impl Hittable for MovingSphere {
    #[inline]
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, Material)> {
        let oc = ray.orig - self.center(ray.time);
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
        let outward_normal = (p - self.center(ray.time)) / self.radius;
        let (u, v) = Sphere::get_uv(&p);
        let mut payload = HitPayload {
            t: root,
            p,
            u,
            v,
            normal: Vec3::ZERO,
            front_face: false,
        };

        payload.set_face_normal(&ray, outward_normal);

        return Some((payload, self.material.clone()));
    }

    fn bounding_box(&self) -> &AABB {
        self.mutate_bbox(self.bounding_box.into());
        return &self.bounding_box;
    }
}

unsafe impl Send for MovingSphere {}
unsafe impl Sync for MovingSphere {}
