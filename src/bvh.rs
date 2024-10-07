use std::cmp::Ordering;

use rand::{thread_rng, Rng};

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::material::MaterialStorage,
    model::Model,
    ray::Ray,
    world::World,
};

#[derive(Debug, Clone)]
pub struct BvhNode {
    left: Model,
    right: Model,
    bbox: AABB,
}

impl BvhNode {
    pub fn new(mut models: Vec<Model>, start: usize, end: usize) -> Model {
        let comp: usize = thread_rng().gen_range(0..2);
        let comp = match comp {
            0 => |a: &Model, b: &Model| compare(a, b, 0),
            1 => |a: &Model, b: &Model| compare(a, b, 1),
            2 => |a: &Model, b: &Model| compare(a, b, 2),
            _ => panic!(),
        };

        let span = end - start;

        let mut node: BvhNode;
        match span {
            1 => {
                node = BvhNode {
                    left: models[start].clone(),
                    right: models[start].clone(),
                    bbox: AABB::default(),
                };
            }
            2 => {
                node = BvhNode {
                    left: models[start].clone(),
                    right: models[start + 1].clone(),
                    bbox: AABB::default(),
                };
            }
            _ => {
                let rest = &mut models[start..end];
                rest.sort_unstable_by(comp);

                let mid = start + span / 2;
                let left = Self::new(models.clone(), start, mid);
                let right = Self::new(models, mid, end);

                node = BvhNode {
                    left,
                    right,
                    bbox: AABB::default(),
                };
            }
        }

        node.bbox = AABB::from((
            node.left.bounding_box().to_owned(),
            node.right.bounding_box().to_owned(),
        ));
        return Model::Bvh(Box::new(node));
    }
    pub fn from_world(world: World) -> Model {
        let len = world.entities.len();
        Self::new(world.entities, 0, len)
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
        if !self.bbox.hit(ray, ray_t) {
            return None;
        }

        let left = self.left.hit(ray, ray_t);
        let interval = Interval::new(
            ray_t.min,
            left.as_ref().map(|(h, _)| h.t).unwrap_or(ray_t.max),
        );
        let right = self.right.hit(ray, interval);
        if right.is_some() {
            right
        } else {
            left
        }
        //right.or(left)
    }

    fn bounding_box(&self) -> &AABB {
        return &self.bbox;
    }
}

fn compare(a: &Model, b: &Model, axis: usize) -> Ordering {
    let a_axis_interval = a.bounding_box().axis(axis);
    let b_axis_interval = b.bounding_box().axis(axis);

    let ac = a_axis_interval.min + a_axis_interval.max;
    let bc = b_axis_interval.min + b_axis_interval.max;
    return ac.partial_cmp(&bc).unwrap();
    //return a_axis_interval.min.total_cmp(&b_axis_interval.min);
}
