pub mod quad;
pub mod sphere;
pub mod transform;
pub mod triangle;
pub mod volume;

use core::panic;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{
    aabb::AABB,
    bvh::{Bvh, BvhNode},
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::MaterialId,
    model::{
        quad::Quad,
        sphere::Sphere,
        transform::{RotateY, Translate},
        volume::ConstantMedium,
    },
    moving_sphere::MovingSphere,
    ray::Ray,
    vec3::Vec3,
    world::World,
    Float,
};

//TODO: improve the size of this, bvh,world etc. don't need to be in here. Quad is also really big,
//might be the main culprit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Model {
    Sphere(Sphere),
    MovingSphere(MovingSphere),
    Quad(Quad),
    ConstantMedium(ConstantMedium),
    #[serde(skip)]
    BvhNode(Box<BvhNode>),
    #[serde(skip)]
    Bvh(Box<Bvh>),
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
    pub fn min(&self, axis: usize) -> Float {
        return self.get_interval(axis).min;
    }

    pub fn max(&self, axis: usize) -> Float {
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
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialId)> {
        match self {
            Model::Sphere(ref m) => m.hit(ray, ray_t),
            Model::MovingSphere(ref m) => m.hit(ray, ray_t),
            Model::Quad(ref m) => m.hit(ray, ray_t),
            Model::BvhNode(ref m) => m.hit(ray, ray_t),
            Model::World(ref m) => m.hit(ray, ray_t),
            Model::Translate(ref m) => m.hit(ray, ray_t),
            Model::RotateY(ref m) => m.hit(ray, ray_t),
            Model::ConstantMedium(ref m) => m.hit(ray, ray_t),
            Model::Shared(ref m) => m.hit(ray, ray_t),
            Model::Bvh(ref m) => m.hit(ray, ray_t),
        }
    }

    fn bounding_box(&self) -> AABB {
        match self {
            Model::Sphere(ref m) => m.bounding_box(),
            Model::MovingSphere(ref m) => m.bounding_box(),
            Model::Quad(ref m) => m.bounding_box(),
            Model::BvhNode(ref m) => m.bounding_box(),
            Model::World(ref m) => m.bounding_box(),
            Model::Translate(ref m) => m.bounding_box(),
            Model::RotateY(ref m) => m.bounding_box(),
            Model::ConstantMedium(ref m) => m.bounding_box(),
            Model::Shared(ref m) => m.bounding_box(),
            Model::Bvh(ref m) => m.bounding_box(),
        }
    }

    fn pdf_value(&self, origin: &Vec3, dir: &Vec3) -> Float {
        match self {
            Model::Sphere(ref m) => m.pdf_value(origin, dir),
            Model::MovingSphere(ref m) => m.pdf_value(origin, dir),
            Model::Quad(ref m) => m.pdf_value(origin, dir),
            Model::BvhNode(ref m) => m.pdf_value(origin, dir),
            Model::World(ref m) => m.pdf_value(origin, dir),
            Model::Translate(ref m) => m.pdf_value(origin, dir),
            Model::RotateY(ref m) => m.pdf_value(origin, dir),
            Model::ConstantMedium(ref m) => m.pdf_value(origin, dir),
            Model::Shared(ref m) => m.pdf_value(origin, dir),
            _ => 0.0,
        }
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        match self {
            Model::Sphere(ref m) => m.random(origin),
            Model::MovingSphere(ref m) => m.random(origin),
            Model::Quad(ref m) => m.random(origin),
            Model::BvhNode(ref m) => m.random(origin),
            Model::World(ref m) => m.random(origin),
            Model::Translate(ref m) => m.random(origin),
            Model::RotateY(ref m) => m.random(origin),
            Model::ConstantMedium(ref m) => m.random(origin),
            Model::Shared(ref m) => m.random(origin),
            _ => Vec3::new(1.0, 0.0, 0.0),
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
        return Self::BvhNode(Box::new(value));
    }
}

impl From<Bvh> for Model {
    fn from(value: Bvh) -> Self {
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
