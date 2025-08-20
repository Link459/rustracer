use crate::{
    aabb::AABB,
    hittable::Hittable,
    material::MaterialId,
    model::{HitPayload, Interval },
    ray::Ray,
    vec3::Vec3,
};

pub struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3,
}

impl Triangle {}

impl Hittable for Triangle {
    #[inline]
    fn hit(&self, ray: &Ray, _ray_t: Interval) -> Option<(HitPayload, MaterialId)> {
        let edge1 = self.b - self.a;
        let edge2 = self.c - self.a;

        let h = ray.dir.cross(&edge2);
        let a = edge1.dot(&h);

        //ray is parallel to the triangle
        if a > -0.00001 && a < 0.0001 {
            return None;
        }

        let f = 1.0 / a;

        let s = ray.orig - self.a;

        let u = f * s.dot(&h);

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(&edge1);
        let v = f * ray.dir.dot(&q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(&q);

        if t > 0.0001 {
            // ray.t = min(ray.t,t);
        }

        return None;
    }

    #[inline]
    fn bounding_box(&self) -> AABB {
        return AABB::EMPTY;
    }
}
