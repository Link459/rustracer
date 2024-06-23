use std::cmp::Ordering;

use rand::Rng;

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::material::Material,
    model::model::Model,
    ray::Ray,
};

/*#[derive(Clone, Debug)]
pub enum BvhNode {
    Branch { left: Box<Bvh>, right: Box<Bvh> },
    Leaf(Model),
}

unsafe impl Send for BvhNode {}
unsafe impl Sync for BvhNode {}

#[derive(Clone, Debug)]
pub struct Bvh {
    pub tree: BvhNode,
    pub hit_box: AABB,
}

impl Bvh {
    pub fn new(world: &mut Vec<Model>, time: Interval) -> Self {
        fn box_compare(axis: usize) -> impl FnMut(&Model, &Model) -> Ordering {
            move |a, b| {
                let a = a.bounding_box();
                let b = b.bounding_box();
                let ac = a.min(axis) + a.max(axis);
                let bc = b.min(axis) + b.max(axis);
                println!("{},{}", ac, bc);
                ac.partial_cmp(&bc).unwrap()
            }
        }

        fn axis_range(world: &Vec<Model>, _time: Interval, axis: usize) -> f64 {
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
unsafe impl Sync for Bvh {}*/

#[derive(Clone, Default, Debug)]
pub struct BvhNode {
    left: Model,
    right: Model,
    bbox: AABB,
}

fn box_compare(a: &Model, b: &Model, axis: usize) -> Ordering {
    let a_i = a.bounding_box().axis(axis);
    let b_i = b.bounding_box().axis(axis);
    return a_i.min.partial_cmp(&b_i.min).unwrap();
}

fn box_compare_x(a: &Model, b: &Model) -> Ordering {
    box_compare(a, b, 0)
}
fn box_compare_y(a: &Model, b: &Model) -> Ordering {
    box_compare(a, b, 1)
}
fn box_compare_z(a: &Model, b: &Model) -> Ordering {
    box_compare(a, b, 2)
}

impl BvhNode {
    pub fn new(world: &mut Vec<Model>, start: usize, end: usize) -> Model {
        let axis = rand::thread_rng().gen_range(0..=2);
        let comp = match axis {
            0 => box_compare_x,
            1 => box_compare_y,
            2 => box_compare_z,
            _ => panic!("should not be able to generate a number greater than 2"),
        };

        let span = end - start;

        let mut node = Self::default();
        if span == 1 {
            node.left = world[start].clone();
            node.right = world[start].clone();
        } else if span == 2 {
            node.left = world[start].clone();
            node.right = world[start + 1].clone();
        } else {
            world.sort_unstable_by(comp);

            let mid = start + span / 2;
            node.left = BvhNode::new(world, start, mid);
            node.right = BvhNode::new(world, mid, end);
        }

        node.bbox = AABB::from((*node.left.bounding_box(), *node.right.bounding_box()));

        return Model::Bvh(Box::new(node));
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, mut ray_t: Interval) -> Option<(HitPayload, Material)> {
        let left = self.left.hit(&ray, ray_t);
        if let Some(l) = &left {
            ray_t.max = l.0.t
        };
        let right = self.right.hit(&ray, ray_t);
        if right.is_some() {
            return right;
        } else {
            return left;
        }
    }

    fn bounding_box(&self) -> &AABB {
        return &self.bbox;
    }
}

/*#[derive(Clone, Debug)]
pub struct BvhNode {
    left: Box<Model>,
    right: Box<Model>,
    bbox: AABB,
}

impl BvhNode {
    pub fn new(bbox: AABB, left: Box<Model>, right: Box<Model>) -> Model {
        return Model::Bvh(BvhNode { left, right, bbox });
    }
    pub fn construct(mut hitable_list: Vec<Model>) -> Model {
        let axis = rand::thread_rng().gen_range(0..3);
        hitable_list.sort_by(|a, b| {
            let a_i = a.bounding_box().axis(axis);
            let b_i = a.bounding_box().axis(axis);
            a_i.min.partial_cmp(&b_i.min).unwrap()
        });
        match hitable_list.len() {
            0 => panic!("length mismatch"),
            1 => hitable_list.remove(0),
            2 => {
                let right = hitable_list.remove(1);
                let left = hitable_list.remove(0);
                let bbox = left.bounding_box().surrounding_box(&right.bounding_box());
                BvhNode::new(bbox, Box::new(left), Box::new(right))
            }
            _ => {
                let mut a = hitable_list;
                let b = a.split_off(a.len() / 2);
                let left = Self::construct(b);
                let right = Self::construct(a);
                let bbox = left.bounding_box().surrounding_box(&right.bounding_box());
                BvhNode::new(bbox, Box::new(left), Box::new(right))
            }
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, Material)> {
        match self.bbox.hit(ray, ray_t) {
            false => None,
            true => {
                let hit_left = self.left.hit(ray, ray_t);
                let hit_right = self.right.hit(ray, ray_t);
                match (hit_left, hit_right) {
                    (None, None) => None,
                    (None, Some(hit_record)) => Some(hit_record),
                    (Some(hit_record), None) => Some(hit_record),
                    (Some(hit_left), Some(hit_right)) => {
                        if hit_left.0.t < hit_right.0.t {
                            Some(hit_left)
                        } else {
                            Some(hit_right)
                        }
                    }
                }
            }
        }
    }

    fn bounding_box(&self) -> &AABB {
        return &self.bbox;
    }
}*/
