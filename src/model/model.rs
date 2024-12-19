use core::panic;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    aabb::AABB,
    bvh::BvhNode,
    hittable::{HitPayload, Hittable, RotateY, Translate},
    interval::Interval,
    material::material::MaterialStorage,
    model::{quad::Quad, sphere::Sphere},
    moving_sphere::MovingSphere,
    ray::Ray,
    volume::ConstantMedium,
    world::World,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Model {
    Sphere(Sphere),
    MovingSphere(MovingSphere),
    Quad(Quad),
    ConstantMedium(ConstantMedium),
    #[serde(skip)]
    Bvh(Box<BvhNode>),
    World(World),
    Translate(Translate),
    RotateY(RotateY),
    #[serde(skip)]
    Shared(Rc<Model>),
}

impl Default for Model {
    fn default() -> Self {
        return Self::World(World::default());
    }
}

impl Model {
    pub fn min(&self, axis: usize) -> f64 {
        return self.get_interval(axis).min;
    }

    pub fn max(&self, axis: usize) -> f64 {
        return self.get_interval(axis).min;
    }

    fn get_interval(&self, axis: usize) -> Interval {
        let bbox = self.bounding_box();
        return match axis {
            0 => bbox.x,
            1 => bbox.y,
            2 => bbox.z,
            _ => panic!("index out of bounds"),
        };
    }
}

impl Hittable for Model {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
        match self {
            Model::Sphere(ref m) => m.hit(ray, ray_t),
            Model::MovingSphere(ref m) => m.hit(ray, ray_t),
            Model::Quad(ref m) => m.hit(ray, ray_t),
            Model::Bvh(ref m) => m.hit(ray, ray_t),
            Model::World(ref m) => m.hit(ray, ray_t),
            Model::Translate(ref m) => m.hit(ray, ray_t),
            Model::RotateY(ref m) => m.hit(ray, ray_t),
            Model::ConstantMedium(ref m) => m.hit(ray, ray_t),
            Model::Shared(ref m) => m.hit(ray, ray_t),
        }
    }

    fn bounding_box(&self) -> AABB {
        match self {
            Model::Sphere(ref m) => m.bounding_box(),
            Model::MovingSphere(ref m) => m.bounding_box(),
            Model::Quad(ref m) => m.bounding_box(),
            Model::Bvh(ref m) => m.bounding_box(),
            Model::World(ref m) => m.bounding_box(),
            Model::Translate(ref m) => m.bounding_box(),
            Model::RotateY(ref m) => m.bounding_box(),
            Model::ConstantMedium(ref m) => m.bounding_box(),
            Model::Shared(ref m) => m.bounding_box(),
        }
    }
}

unsafe impl Sync for Model {}
unsafe impl Send for Model {}
