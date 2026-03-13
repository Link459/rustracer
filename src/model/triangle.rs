use crate::{
    aabb::AABB,
    hittable::Hittable,
    material::MaterialId,
    model::{HitPayload, Interval},
    ray::Ray,
    vec3::Vec3,
};

pub struct Triangle {
    pos1: Vec3,
    pos2: Vec3,
    pos3: Vec3,
    material: MaterialId,
}

impl Triangle {}

impl Hittable for Triangle {
    #[inline]
    fn hit(&self, ray: &Ray, _ray_t: Interval) -> Option<(HitPayload, MaterialId)> {
        let edge1 = self.pos2 - self.pos1;
        let edge2 = self.pos3 - self.pos1;

        let normal = edge1.cross(&edge2);

        let det = -ray.dir.dot(&normal);
        let inv_det = 1.0 / det;

        let ao = ray.orig - self.pos1;
        let dao = ao.cross(&ray.dir);

        let u = edge2.dot(&dao) * inv_det;
        let v = -edge1.dot(&dao) * inv_det;
        let t = ao.dot(&normal) * inv_det;

        let hit = det >= 1e-6 && t >= 0.0 && u >= 0.0 && v >= 0.0 && (u + v) <= 1.0;
        if hit {
            let intersection = ray.orig + t * ray.dir;
            let payload = HitPayload {
                p: intersection,
                normal,
                t,
                u,
                v,
                front_face: true,
            };
            return Some((payload, self.material));
        }
        return None;
    }

    #[inline]
    fn bounding_box(&self) -> AABB {
        return AABB::EMPTY;
    }
}
