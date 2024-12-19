use std::{
    fmt::{self, Display, Formatter},
    write,
};

use serde::{Deserialize, Serialize};

use crate::{
    aabb::AABB, interval::Interval, material::material::MaterialStorage, model::model::Model,
    ray::Ray, vec3::Vec3,
};

#[derive(Default, Debug)]
pub struct HitPayload {
    pub p: Vec3,
    pub normal: Vec3,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

impl HitPayload {
    pub fn new(p: Vec3, normal: Vec3, t: f64, u: f64, v: f64) -> Self {
        Self {
            p,
            normal,
            t,
            u,
            v,
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) -> () {
        self.front_face = Vec3::dot(&ray.dir, &outward_normal) < 0.0;
        self.normal = match self.front_face {
            true => outward_normal,
            false => -outward_normal,
        }
    }
}

impl Display for HitPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.p)?;
        write!(f, "{:?}", self.normal)?;
        write!(f, "{:?}", self.t)?;
        write!(f, "{:?}", self.u)?;
        write!(f, "{:?}", self.v)?;
        write!(f, "{:?}", self.front_face)?;
        Ok(())
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)>;
    fn bounding_box(&self) -> AABB;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Translate {
    model: Box<Model>,
    offset: Vec3,
}

impl Translate {
    pub fn new(model: Box<Model>, offset: Vec3) -> Model {
        Model::Translate(Self { offset, model })
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
    sin_theta: f64,
    cos_theta: f64,
}

impl RotateY {
    pub fn new(model: Box<Model>, angle: f64) -> Model {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();


        return Model::RotateY(Self {
            model,
            sin_theta,
            cos_theta,
        });
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
        let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Vec3::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f64 * bbox.x.max + (1.0 - i as f64) * bbox.x.min;
                    let y = j as f64 * bbox.y.max + (1.0 - j as f64) * bbox.y.min;
                    let z = k as f64 * bbox.z.max + (1.0 - k as f64) * bbox.z.min;

                    let newx = self.cos_theta * x + self.sin_theta * z;
                    let newz = -self.sin_theta * x + self.cos_theta * z;

                    let tester = Vec3::new(newx, y, newz);

                    for c in 0..3 {
                        min[c] = f64::min(min[c], tester[c]);
                        max[c] = f64::max(max[c], tester[c]);
                    }
                }
            }
        }

        return AABB::from((min, max));
    }
}
