use std::{cmp::Ordering, collections::VecDeque, fmt::Display, usize};

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::MaterialId,
    model::Model,
    ray::Ray,
    world::World,
};

#[derive(Default, Debug, Clone, Copy)]
pub struct Node {
    pub bbox: AABB,
    pub primitive_count: u32,
    pub first_idx: u32,
}

impl Node {
    fn is_leaf(&self) -> bool {
        return self.primitive_count != 0;
    }
}

#[derive(Clone, Debug)]
pub struct Bvh {
    nodes: Vec<Node>,
    prim_indices: Vec<usize>,
    models: Vec<Model>,
}

impl Bvh {
    pub fn new(nodes: Vec<Node>, prim_indices: Vec<usize>, models: Vec<Model>) -> Self {
        Self {
            nodes,
            prim_indices,
            models,
        }
    }
}

impl Hittable for Bvh {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialId)> {
        let mut stack = VecDeque::new();
        stack.push_back(0);

        while !stack.is_empty() {
            let node = &self.nodes[stack.pop_front().unwrap()];

            if !node.bbox.hit(ray, ray_t) {
                continue;
            }

            if node.is_leaf() {
                // we keep track of the most current hit and check the distance to ensure that only
                // the primitive nearest to the camera is returned
                let mut current: Option<(HitPayload, MaterialId)> = None;

                for i in 0..node.primitive_count {
                    let prim_index = self.prim_indices[(node.first_idx + i) as usize];

                    let model = &self.models[prim_index];

                    if let Some(x) = model.hit(ray, ray_t) {
                        if let Some(ref y) = current {
                            if x.0.t < y.0.t {
                                current = Some(x);
                            }
                        } else {
                            current = Some(x);
                        }
                    }
                }

                return current;
            } else {
                stack.push_back(node.first_idx as usize);
                stack.push_back(node.first_idx as usize + 1);
            }
        }
        None
    }

    fn bounding_box(&self) -> AABB {
        return self.nodes[0].bbox;
    }
}

impl Display for Bvh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stack = VecDeque::new();
        stack.push_back(0);

        while !stack.is_empty() {
            let node = &self.nodes[stack.pop_front().unwrap()];

            if node.is_leaf() {
                for i in 0..node.primitive_count {
                    let prim_index = self.prim_indices[(node.first_idx + i) as usize];

                    let model = &self.models[prim_index];
                    writeln!(f, "{:?}", model)?;
                }
            } else {
                writeln!(f, "left: {}", node.first_idx)?;
                writeln!(f, "right: {}", node.first_idx + 1)?;

                stack.push_back(node.first_idx as usize);
                stack.push_back(node.first_idx as usize + 1);
            }
        }
        return Ok(());
    }
}

#[derive(Clone, Debug)]
pub struct BvhNode {
    left: Model,
    right: Model,
    bbox: AABB,
}

impl BvhNode {
    pub fn new(mut models: Vec<Model>, start: usize, end: usize) -> Self {
        let mut bbox = AABB::EMPTY;

        for model in models.iter().take(end).skip(start) {
            bbox = AABB::from((bbox, model.bounding_box()));
        }

        let axis = bbox.longest_axis();
        let comp = match axis {
            0 => |a: &Model, b: &Model| compare(a, b, 0),
            1 => |a: &Model, b: &Model| compare(a, b, 1),
            2 => |a: &Model, b: &Model| compare(a, b, 2),
            _ => panic!(),
        };

        let span = end - start;

        let node = match span {
            1 => BvhNode {
                left: models[start].clone(),
                right: models[start].clone(),
                bbox,
            },
            2 => BvhNode {
                left: models[start].clone(),
                right: models[start + 1].clone(),
                bbox,
            },
            _ => {
                let rest = &mut models[start..end];
                rest.sort_unstable_by(comp);

                let mid = start + span / 2;
                let left = Self::new(models.clone(), start, mid);
                let right = Self::new(models, mid, end);

                BvhNode {
                    left: left.into(),
                    right: right.into(),
                    bbox,
                }
            }
        };

        return node;
    }

    pub fn from_world(world: World) -> Self {
        let len = world.entities.len();
        Self::new(world.entities, 0, len)
    }
}

impl Hittable for BvhNode {
    #[inline(always)]
    fn hit(&self, ray: &Ray, mut ray_t: Interval) -> Option<(HitPayload, MaterialId)> {
        if !self.bbox.hit(ray, ray_t) {
            return None;
        }

        let left = self.left.hit(ray, ray_t);
        if left.is_some() {
            unsafe {
                ray_t.max = left.as_ref().unwrap_unchecked().0.t;
            }
        }
        let right = self.right.hit(ray, ray_t);
        if right.is_some() {
            right
        } else {
            left
        }
    }

    fn bounding_box(&self) -> AABB {
        return self.bbox;
    }
}

fn compare(a: &Model, b: &Model, axis: usize) -> Ordering {
    let a_axis_interval = a.bounding_box().axis(axis);
    let b_axis_interval = b.bounding_box().axis(axis);

    let ac = a_axis_interval.min + a_axis_interval.max;
    let bc = b_axis_interval.min + b_axis_interval.max;
    return ac.total_cmp(&bc);
    //return ac.partial_cmp(&bc).unwrap();
}
