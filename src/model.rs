use core::panic;

use crate::{
    aabb::AABB,
    bvh::Bvh,
    hittable::Hittable,
    hittable::{HitPayload, RotateY, Translate},
    interval::Interval,
    material::Material,
    moving_sphere::MovingSphere,
    quad::Quad,
    ray::Ray,
    sphere::Sphere,
    volume::ConstantMedium,
    world::World,
};

#[derive(Clone)]
pub enum Model {
    Sphere(Sphere),
    MovingSphere(MovingSphere),
    Quad(Quad),
    ConstantMedium(ConstantMedium),
    Bvh(Box<Bvh>),

    World(World),
    Translate(Translate),
    RotateY(RotateY),
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
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, Material)> {
        match self {
            Model::Sphere(ref m) => m.hit(ray, ray_t),
            Model::MovingSphere(ref m) => m.hit(ray, ray_t),
            Model::Quad(ref m) => m.hit(ray, ray_t),
            Model::Bvh(ref m) => m.hit(ray, ray_t),
            Model::World(ref m) => m.hit(ray, ray_t),
            Model::Translate(ref m) => m.hit(ray, ray_t),
            Model::RotateY(ref m) => m.hit(ray, ray_t),
            Model::ConstantMedium(ref m) => m.hit(ray, ray_t),
        }
    }

    fn bounding_box(&self) -> &AABB {
        match self {
            Model::Sphere(ref m) => m.bounding_box(),
            Model::MovingSphere(ref m) => m.bounding_box(),
            Model::Quad(ref m) => m.bounding_box(),
            Model::Bvh(ref m) => m.bounding_box(),
            Model::World(ref m) => m.bounding_box(),
            Model::Translate(ref m) => m.bounding_box(),
            Model::RotateY(ref m) => m.bounding_box(),
            Model::ConstantMedium(ref m) => m.bounding_box(),
        }
    }
}

unsafe impl Sync for Model {}
unsafe impl Send for Model {}
