use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::{material::MaterialStorage, Isotropic},
    model::Model,
    ray::Ray,
    vec3::Vec3,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstantMedium {
    boundary: Box<Model>,
    neg_inv_density: f64,
    phase_func: MaterialStorage,
}

impl ConstantMedium {
    pub fn new(boundary: Box<Model>, d: f64, c: Vec3) -> Model {
        Model::ConstantMedium(Self {
            boundary,
            neg_inv_density: -1.0 / d,
            phase_func: Isotropic::from(c).into(),
        })
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
        let mut rng = rand::thread_rng();
        if let Some((mut hit1, _)) = self.boundary.hit(&ray, Interval::new(-f64::MAX, f64::MAX)) {
            if let Some((mut hit2, _)) = self
                .boundary
                .hit(&ray, Interval::new(hit1.t + 0.0001, f64::MAX))
            {
                if hit1.t < ray_t.min {
                    hit1.t = ray_t.min
                }
                if hit2.t > ray_t.max {
                    hit2.t = ray_t.max
                }
                if hit1.t < hit2.t {
                    let distance_inside_boundary = (hit2.t - hit1.t) * ray.dir.length();
                    let hit_distance = self.neg_inv_density * rng.gen::<f64>().ln();
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
