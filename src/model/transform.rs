use crate::{
    aabb::AABB,
    hittable::Hittable,
    model::{HitPayload, Interval, MaterialStorage, Model},
    ray::Ray,
    vec3::Vec3, Float,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Translate {
    model: Box<Model>,
    offset: Vec3,
}

impl Translate {
    pub fn new(model: impl Into<Model>, offset: Vec3) -> Self {
        return Self {
            offset,
            model: Box::new(model.into()),
        };
    }
}

impl Hittable for Translate {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
        let offset_ray = Ray::new(ray.orig - self.offset, ray.dir, ray.time);

        if let Some((mut payload, material)) = self.model.hit(&offset_ray, ray_t) {
            payload.p += self.offset;
            return Some((payload, material));
        }

        return None;
    }

    fn bounding_box(&self) -> AABB {
        //return &self.bbox;
        return self.model.bounding_box() + self.offset;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RotateY {
    model: Box<Model>,
    sin_theta: Float,
    cos_theta: Float,
}

impl RotateY {
    pub fn new(model: impl Into<Model>, angle: Float) -> Self {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        return Self {
            model: Box::new(model.into()),
            sin_theta,
            cos_theta,
        };
    }
}

impl Hittable for RotateY {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
        let mut origin = ray.orig;
        let mut direction = ray.dir;

        origin[0] = self.cos_theta * ray.orig[0] - self.sin_theta * ray.orig[2];
        origin[2] = self.sin_theta * ray.orig[0] + self.cos_theta * ray.orig[2];

        direction[0] = self.cos_theta * ray.dir[0] - self.sin_theta * ray.dir[2];
        direction[2] = self.sin_theta * ray.dir[0] + self.cos_theta * ray.dir[2];

        let rotated_ray = Ray::new(origin, direction, ray.time);

        // Determine where (if any) an intersection occurs in object space
        if let Some((mut payload, material)) = self.model.hit(&rotated_ray, ray_t) {
            // Change the intersection point from object space to world space
            let mut p = payload.p;
            p[0] = self.cos_theta * payload.p[0] + self.sin_theta * payload.p[2];
            p[2] = -self.sin_theta * payload.p[0] + self.cos_theta * payload.p[2];

            // Change the normal from object space to world space
            let mut normal = payload.normal;
            normal[0] = self.cos_theta * payload.normal[0] + self.sin_theta * payload.normal[2];
            normal[2] = -self.sin_theta * payload.normal[0] + self.cos_theta * payload.normal[2];

            payload.p = p;
            payload.normal = normal;

            return Some((payload, material));
        }

        return None;
    }

    fn bounding_box(&self) -> AABB {
        let bbox = self.model.bounding_box();
        let mut min = Vec3::new(Float::INFINITY, Float::INFINITY, Float::INFINITY);
        let mut max = Vec3::new(-Float::INFINITY, -Float::INFINITY, -Float::INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as Float * bbox.x.max + (1.0 - i as Float) * bbox.x.min;
                    let y = j as Float * bbox.y.max + (1.0 - j as Float) * bbox.y.min;
                    let z = k as Float * bbox.z.max + (1.0 - k as Float) * bbox.z.min;

                    let newx = self.cos_theta * x + self.sin_theta * z;
                    let newz = -self.sin_theta * x + self.cos_theta * z;

                    let tester = Vec3::new(newx, y, newz);

                    for c in 0..3 {
                        min[c] = Float::min(min[c], tester[c]);
                        max[c] = Float::max(max[c], tester[c]);
                    }
                }
            }
        }

        return AABB::from((min, max));
    }
}
