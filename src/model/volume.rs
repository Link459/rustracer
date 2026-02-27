use rand::{ RngExt};
use serde::{Deserialize, Serialize};

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::{Isotropic, MaterialId, MaterialStore},
    model::Model,
    ray::Ray,
    vec3::Vec3,
    Float,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstantMedium {
    boundary: Box<Model>,
    neg_inv_density: Float,
    phase_func: MaterialId,
}

impl ConstantMedium {
    pub fn new(boundary: impl Into<Model>, d: Float, c: Vec3, store: &mut MaterialStore) -> Self {
        return Self {
            boundary: Box::new(boundary.into()),
            neg_inv_density: -1.0 / d,
            phase_func: store.add(Isotropic::from(c)),
        };
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialId)> {
        let mut rng = rand::rng();
        if let Some((mut hit1, _)) = self
            .boundary
            .hit(ray, Interval::new(-Float::MAX, Float::MAX))
        {
            if let Some((mut hit2, _)) = self
                .boundary
                .hit(ray, Interval::new(hit1.t + 0.0001, Float::MAX))
            {
                if hit1.t < ray_t.min {
                    hit1.t = ray_t.min
                }
                if hit2.t > ray_t.max {
                    hit2.t = ray_t.max
                }
                if hit1.t < hit2.t {
                    let distance_inside_boundary = (hit2.t - hit1.t) * ray.dir.length();
                    let hit_distance = self.neg_inv_density * rng.random::<Float>().ln();
                    if hit_distance < distance_inside_boundary {
                        let t = hit1.t + hit_distance / ray.dir.length();
                        return Some((
                            HitPayload {
                                t,
                                u: 0.0,
                                v: 0.0,
                                p: ray.at(t),
                                normal: Vec3::new(1.0, 0.0, 0.0), // arbitrary
                                front_face: false,
                            },
                            self.phase_func.clone(),
                        ));
                    }
                }
            }
        }
        None
    }

    fn bounding_box(&self) -> AABB {
        return self.boundary.bounding_box();
    }
}
