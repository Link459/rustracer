pub mod quad;
pub mod sphere;

use core::panic;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    aabb::AABB,
    bvh::BvhNode,
    hittable::{HitPayload, Hittable, RotateY, Translate},
    interval::Interval,
    material::MaterialStorage,
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

impl From<Sphere> for Model {
    fn from(value: Sphere) -> Self {
        return Self::Sphere(value);
    }
}

impl From<MovingSphere> for Model {
    fn from(value: MovingSphere) -> Self {
        return Self::MovingSphere(value);
    }
}

impl From<Quad> for Model {
    fn from(value: Quad) -> Self {
        return Self::Quad(value);
    }
}

impl From<BvhNode> for Model {
    fn from(value: BvhNode) -> Self {
        return Self::Bvh(Box::new(value));
    }
}

impl From<World> for Model {
    fn from(value: World) -> Self {
        return Self::World(value);
    }
}

impl From<Translate> for Model {
    fn from(value: Translate) -> Self {
        return Self::Translate(value);
    }
}

impl From<RotateY> for Model {
    fn from(value: RotateY) -> Self {
        return Self::RotateY(value);
    }
}

impl From<ConstantMedium> for Model {
    fn from(value: ConstantMedium) -> Self {
        return Self::ConstantMedium(value);
    }
}
