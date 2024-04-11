use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::Material,
    model::Model,
    ray::Ray,
    vec3::Vec3,
};

#[derive(Clone)]
pub struct Quad {
    q: Vec3,
    u: Vec3,
    v: Vec3,
    material: Material,
    bbox: AABB,
    normal: Vec3,
    d: f64,
    w: Vec3,
}

impl Quad {
    pub fn new(q: Vec3, u: Vec3, v: Vec3, material: Material) -> Model {
        let n = u.cross(&v);
        let normal = n.normalize();
        let d = normal.dot(&q);

        let bbox = AABB::from((q, q + u + v)).pad();
        let w = n / n.dot(&n);

        Model::Quad(Self {
            q,
            u,
            v,
            material,
            bbox,
            normal,
            d,
            w,
        })
    }

    fn is_interior(a: f64, b: f64) -> Option<(f64, f64)> {
        // Given the hit point in plane coordinates, return false if it is outside the
        // primitive, otherwise set the hit record UV coordinates and return true.

        if a < 0.0 || 1.0 < a || b < 0.0 || 1.0 < b {
            return None;
        }

        return Some((a, b));
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, Material)> {
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
            payload.set_face_normal(&ray, self.normal);

            return Some((payload, self.material.clone()));
        }
        return None;
    }

    fn bounding_box(&self) -> &AABB {
        return &self.bbox;
    }
}
