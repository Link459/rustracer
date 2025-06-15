use std::{cmp::Ordering, collections::VecDeque};

use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
    interval::Interval,
    material::MaterialStorage,
    model::Model,
    ray::Ray,
    vec3::Vec3,
    world::World,
};

pub struct BvhBuildConfig {
    min_prims: usize,
    max_prims: usize,
    traversal_cost: f32,
}

impl Default for BvhBuildConfig {
    fn default() -> Self {
        Self {
            min_prims: 2,
            max_prims: 8,
            traversal_cost: 1.0,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Node {
    bbox: AABB,
    primitive_count: u32,
    first_idx: u32,
}

impl Node {
    fn is_leaf(&self) -> bool {
        return self.primitive_count != 0;
    }
}

#[derive(Default, Clone, Copy)]
struct Bin {
    bbox: AABB,
    primitive_count: usize,
}

impl Bin {
    fn extend(&mut self, other: &Bin) {
        self.bbox = AABB::from((self.bbox, other.bbox));
        self.primitive_count += other.primitive_count;
    }

    fn cost(&self) -> f64 {
        return self.bbox.half_area() * self.primitive_count as f64;
    }
}

const BIN_COUNT: usize = 16;

fn bin_index(axis: usize, bbox: AABB, center: Vec3) -> usize {
    let index = (center.axis(axis) - bbox.min_axis(axis))
        * (BIN_COUNT as f64 / (bbox.max_axis(axis) - bbox.min_axis(axis)));
    return (BIN_COUNT - 1).min((0f64.max(index)) as usize);
}

struct Split {
    axis: usize,
    cost: f64,
    right_bin: usize,
}

impl Split {
    pub fn find_best_split(axis: usize, bvh: &Bvh, node_idx: usize) -> Split {
        let node = &bvh.nodes[node_idx];
        let mut bins = [Bin::default(); BIN_COUNT];

        for i in 0..node.primitive_count {
            let prim_idx = node.first_idx + i;
            let bin = &mut bins[bin_index(axis, node.bbox, node.bbox.center())];

            let model = &bvh.models[bvh.prim_indices[prim_idx as usize]];
            bin.bbox = AABB::from((bin.bbox, model.bounding_box()));
            bin.primitive_count += 1;
        }

        let mut right_cost = [0.0; BIN_COUNT];
        let mut left_accum = Bin::default();
        let mut right_accum = Bin::default();

        for i in (0..BIN_COUNT).rev() {
            right_accum.extend(&bins[i]);
            right_cost[i] = right_accum.cost();
        }

        let mut split = Split {
            axis,
            cost: f64::MAX,
            right_bin: 0,
        };

        for i in 0..(BIN_COUNT - 1) {
            left_accum.extend(&bins[i]);
            let cost = left_accum.cost() + right_cost[i + 1];

            if cost < split.cost {
                split.cost = cost;
                split.right_bin = i + 1;
            }
        }
        return split;
    }
}

#[derive(Clone, Debug)]
pub struct Bvh {
    nodes: Vec<Node>,
    prim_indices: Vec<usize>,
    models: Vec<Model>,
}

impl Bvh {
    pub fn new(models: Vec<Model>) -> Self {
        let primitive_count = models.len();
        let nodes = Vec::new();
        let prim_indices = Vec::new();
        let mut bvh = Bvh {
            nodes,
            prim_indices,
            models,
        };

        bvh.build(primitive_count);
        return bvh;
    }

    fn build(&mut self, prim_count: usize) {
        self.prim_indices = (0..prim_count).collect::<Vec<_>>();

        let node_count = 2 * prim_count - 1;
        self.nodes.resize_with(node_count, || Node::default());
        self.nodes[0].primitive_count = prim_count as u32;
        self.nodes[0].first_idx = 0;

        let mut node_count = 1;
        self.build_recursive(0, &mut node_count);
    }

    fn build_recursive(&mut self, node_idx: usize, node_count: &mut usize) {
        {
            let node = &mut self.nodes[node_idx];
            assert!(node.is_leaf());

            node.bbox = AABB::EMPTY;

            if node.primitive_count <= 2 {
                return;
            }

            for i in 0..node.primitive_count {
                let model = &self.models[self.prim_indices[(node.first_idx + i) as usize]];
                node.bbox = AABB::from((node.bbox, model.bounding_box()));
            }
        }

        let mut min_split = Split {
            axis: 0,
            cost: f64::MAX,
            right_bin: 0,
        };

        for axis in 0..3 {
            let best_split = Split::find_best_split(axis, &self, node_idx);
            if min_split.cost < best_split.cost {
                min_split = best_split;
            }
        }

        let node = &mut self.nodes[node_idx];
        //config.traversal_cost
        let leaf_cost = node.bbox.half_area() * (node.primitive_count - 1) as f64;
        let mut first_right = 0;

        if min_split.right_bin == 0 || min_split.cost >= leaf_cost {
            //config.max_primitives
            if node.primitive_count > 8 {
                /*let axis = node.bbox.longest_axis();
                //sort primitives
                let start = node.first_idx as usize;
                let end = start + node.primitive_count as usize;

                let rest = &mut self.prim_indices[start..end];
                let get_center = |u: usize| self.nodes[u].bbox.center().axis(axis);
                rest.sort_unstable_by(|a, b| get_center(*a).partial_cmp(&get_center(*b)).unwrap());*/

                first_right = node.first_idx + node.primitive_count / 2;
            } else {
                //Terminate as it's a leaf
                return;
            }
        } else {
            // good split, partition the primitives

            /*let start = node.first_idx as usize;
            let end = start + node.primitive_count as usize;

            let rest = &mut self.prim_indices[start..end];
            rest.iter_mut().partition_in_place(|i| {
                let center = self.nodes[*i].bbox.center();
                bin_index(min_split.axis, node.bbox, center) < min_split.right_bin
            });*/
        }

        let first_child = *node_count;

        *node_count += 2;

        let left_prim_count = first_right - node.first_idx;
        let right_prim_count = node.primitive_count - left_prim_count;

        let left_first_index = node.first_idx;
        let right_first_index = first_right;

        let left = &mut self.nodes[first_child];
        left.primitive_count = left_prim_count;
        left.first_idx = left_first_index;

        let right = &mut self.nodes[first_child + 1];
        right.primitive_count = right_prim_count;
        right.first_idx = right_first_index;

        self.build_recursive(first_child, node_count);
        self.build_recursive(first_child + 1, node_count);
    }
}

impl Hittable for Bvh {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
        let mut stack = VecDeque::new();
        stack.push_back(0);

        while !stack.is_empty() {
            let node = &self.nodes[*stack.front().unwrap()];
            stack.pop_front();

            if !node.bbox.hit(ray, ray_t) {
                continue;
            }

            if node.is_leaf() {
                for i in 0..node.primitive_count {
                    let prim_index = self.prim_indices[(node.first_idx + i) as usize];

                    let model = &self.models[prim_index];

                    if let Some((payload, material)) = model.hit(ray, ray_t) {
                        return Some((payload, material));
                    }
                }
            } else {
                stack.push_back(node.first_idx as usize);
                stack.push_back(node.first_idx as usize + 1);
            }
        }
        None
    }
    fn bounding_box(&self) -> AABB {
        return AABB::EMPTY;
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
    fn hit(&self, ray: &Ray, mut ray_t: Interval) -> Option<(HitPayload, MaterialStorage)> {
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
