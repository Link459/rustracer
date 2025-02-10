use core::{f32, f64};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::MaterialStorage,
    ray::Ray,
    vec3::Vec3,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quad {
    q: Vec3,
    u: Vec3,
    v: Vec3,
    material: MaterialStorage,
    #[serde(skip)]
    bbox: AABB,
    normal: Vec3,
    d: f64,
    w: Vec3,
    area: f64,
}

impl Quad {
    pub fn new(q: Vec3, u: Vec3, v: Vec3, material: impl Into<MaterialStorage>) -> Self {
        let n = u.cross(&v);
        let normal = n.normalize();
        let d = normal.dot(&q);

        let bbox = AABB::from((q, q + u + v)).pad();
        let w = n / n.dot(&n);
        let area = n.length();

        return Self {
            q,
            u,
            v,
            material: material.into(),
            bbox,
            normal,
            d,
            w,
            area,
        };
    }

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)> {
        // Given the hit point in plane coordinates, return false if it is outside the
        // primitive, otherwise set the hit record UV coordinates and return true.

        if !(0.0..=1.0).contains(&a) || !(0.0..=1.0).contains(&b)
        //if a < 0.0 || 1.0 < a || b < 0.0 || 1.0 < b
        {
            return None;
        }

        return Some((a, b));
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
        let denom = self.normal.dot(&ray.dir);

        // No hit if the ray is parallel to the plane.
        if denom.abs() < 1e-8 {
            return None;
        }

        // Return false if the hit point parameter t is outside the ray interval.
        let t = (self.d - self.normal.dot(&ray.orig)) / denom;
        if !ray_t.contains(t) {
            return None;
        }

        let intersection = ray.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let alpha = self.w.dot(&planar_hitpt_vector.cross(&self.v));
        let beta = self.w.dot(&self.u.cross(&planar_hitpt_vector));

        if let Some((u, v)) = Self::is_interior(alpha, beta) {
            let mut payload = HitPayload::new(intersection, self.normal, t, u, v);
            payload.set_face_normal(ray, self.normal);

            return Some((payload, self.material.clone()));
        }
        return None;
    }

    fn bounding_box(&self) -> AABB {
        let bbox = AABB::from((self.q, self.q + self.u + self.v)).pad();
        return bbox;
    }

    fn pdf_value(&self, origin: &Vec3, dir: &Vec3) -> f64 {
        let ray = Ray::new(*origin, *dir, 0.0);
        let Some((payload, _material)) = self.hit(&ray, Interval::new(0.001, f64::INFINITY)) else {
            return 0.0;
        };

        let distance_sq = payload.t * payload.t * dir.length_squared();
        let cosine = dir.dot(&payload.normal).abs() / dir.length();
        return distance_sq / (cosine * self.area);
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let mut rng = thread_rng();
        let p = self.q + (rng.gen_range(0.0..1.0) * self.u) + (rng.gen_range(0.0..1.0) * self.v);
        return p - origin;
    }
}
