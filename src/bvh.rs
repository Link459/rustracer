use std::cmp::Ordering;

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::Material,
    model::Model,
    ray::Ray,
};

#[derive(Clone)]
pub enum BvhNode {
    Branch { left: Box<Bvh>, right: Box<Bvh> },
    Leaf(Model),
}
unsafe impl Send for BvhNode {}
unsafe impl Sync for BvhNode {}

#[derive(Clone)]
pub struct Bvh {
    pub tree: BvhNode,
    pub hit_box: AABB,
}

impl Bvh {
    pub fn new(world: &mut Vec<Model>, time: Interval) -> Self {
        fn box_compare(axis: usize) -> impl FnMut(&Model, &Model) -> Ordering {
            move |a, b| {
                let a_hit_box = a.bounding_box();
                let b_hit_box = b.bounding_box();
                let ac = a.min(axis) + a.max(axis);
                let bc = b.min(axis) + b.max(axis);
                ac.partial_cmp(&bc).unwrap()
            }
        }

        fn axis_range(world: &Vec<Model>, time: Interval, axis: usize) -> f64 {
            let (min, max) = world
                .iter()
                .fold((f64::MAX, f64::MIN), |(bmin, bmax), hit| {
                    //if let Some(aabb) = hit.bounding_box() {
                    let aabb = hit.bounding_box();
                    (bmin.min(aabb.min(axis)), bmax.max(aabb.max(axis)))
                    /*} else {
                        (bmin, bmax)
                    }*/
                });
            max - min
        }

        let mut axis_ranges: Vec<(usize, f64)> =
            (0..3).map(|a| (a, axis_range(&world, time, a))).collect();

        axis_ranges.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let axis = axis_ranges[0].0;

        world.sort_unstable_by(box_compare(axis));
        let len = world.len();
        match len {
            0 => panic!["no elements in scene"],
            1 => {
                let leaf = world.pop().unwrap();
                //if let Some(hit_box) = leaf.bounding_box() {
                let hit_box = leaf.bounding_box();
                Bvh {
                    tree: BvhNode::Leaf(leaf.clone()),
                    hit_box: *hit_box,
                }
                /*} else {
                    panic!["no bounding box in bvh node"]
                }*/
            }
            _ => {
                let right = Bvh::new(&mut world.drain(len / 2..).collect::<Vec<Model>>(), time);
                let left = Bvh::new(world, time);
                let hit_box = AABB::surrounding_box(&left.hit_box, &right.hit_box);
                return Bvh {
                    tree: BvhNode::Branch {
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    hit_box,
                };
            }
        }
    }
}

impl Hittable for Bvh {
    fn hit(&self, ray: &Ray, mut ray_t: Interval) -> Option<(HitPayload, Material)> {
        if self.hit_box.hit(&ray, ray_t) {
            match &self.tree {
                BvhNode::Leaf(leaf) => leaf.hit(&ray, ray_t),
                BvhNode::Branch { left, right } => {
                    let left = left.hit(&ray, ray_t);
                    if let Some(l) = &left {
                        ray_t.max = l.0.t
                    };
                    let right = right.hit(&ray, ray_t);
                    if right.is_some() {
                        return right;
                    } else {
                        return left;
                    }
                }
            }
        } else {
            return None;
        }
    }

    fn bounding_box(&self) -> &AABB {
        return &self.hit_box;
    }
}

unsafe impl Send for Bvh {}
unsafe impl Sync for Bvh {}
