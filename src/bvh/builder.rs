use super::Node;
use crate::aabb::AABB;
use crate::hittable::Hittable;
use crate::vec3::Vec3;
use crate::Float;
use crate::{bvh::Bvh, model::Model, world::World};

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

    fn cost(&self) -> Float {
        return self.bbox.half_area() * self.primitive_count as Float;
    }
}

fn bin_index(bin_count: usize, axis: usize, bbox: AABB, center: Vec3) -> usize {
    let index = (center.axis(axis) - bbox.min_axis(axis))
        * ((bin_count as Float) / (bbox.max_axis(axis) - bbox.min_axis(axis)));
    let null: Float = 0.0;
    let idx = null.max(index);
    return (bin_count - 1).min((idx) as usize);
}

struct Split {
    axis: usize,
    cost: Float,
    right_bin: usize,
}

impl Split {
    pub fn find_best_split(axis: usize, bvh: &BvhBuilder, node_idx: usize) -> Split {
        let node = &bvh.nodes[node_idx];
        let mut bins = Vec::new();
        bins.resize(bvh.bin_count, Bin::default());

        for i in 0..node.primitive_count {
            let prim_idx = node.first_idx + i;
            let bin = &mut bins[bin_index(bvh.bin_count, axis, node.bbox, node.bbox.center())];

            let model = &bvh.models[bvh.prim_indices[prim_idx as usize]];
            bin.bbox = AABB::from((bin.bbox, model.bounding_box()));
            bin.primitive_count += 1;
        }

        let mut right_cost = Vec::new();
        right_cost.resize(bvh.bin_count, 0.0);
        let mut left_accum = Bin::default();
        let mut right_accum = Bin::default();

        for i in (0..bvh.bin_count).rev() {
            right_accum.extend(&bins[i]);
            right_cost[i] = right_accum.cost();
        }

        let mut split = Split {
            axis,
            cost: Float::MAX,
            right_bin: 0,
        };

        for i in 0..(bvh.bin_count - 1) {
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

pub struct BvhBuilder {
    nodes: Vec<Node>,
    prim_indices: Vec<usize>,
    models: Vec<Model>,

    min_prims: usize,
    max_prims: usize,
    bin_count: usize,
    traversal_cost: Float,
}

impl BvhBuilder {
    pub fn new(models: Vec<Model>) -> Self {
        let nodes = Vec::new();
        let prim_indices = Vec::new();
        return Self {
            nodes,
            prim_indices,
            models,
            min_prims: 2,
            max_prims: 8,
            bin_count: 16,
            traversal_cost: 1.0,
        };
    }

    pub fn from_world(world: World) -> Self {
        return Self::new(world.entities);
    }

    pub fn min_primitives(mut self, min: usize) -> Self {
        self.min_prims = min;
        return self;
    }

    pub fn max_primitives(mut self, max: usize) -> Self {
        self.max_prims = max;
        return self;
    }

    pub fn bin_count(mut self, bins: usize) -> Self {
        self.bin_count = bins;
        return self;
    }

    pub fn traversal_cost(mut self, cost: Float) -> Self {
        self.traversal_cost = cost;
        return self;
    }

    pub fn build(mut self) -> Bvh {
        let prim_count = self.models.len();
        self.prim_indices = (0..prim_count).collect::<Vec<_>>();

        let node_count = 2 * prim_count - 1;
        self.nodes.resize_with(node_count, || Node::default());
        let root = &mut self.nodes[0];
        root.primitive_count = prim_count as u32;
        root.first_idx = 0;

        let mut node_count = 1;
        self.build_recursive(0, &mut node_count);

        return Bvh::new(self.nodes, self.prim_indices, self.models);
    }

    fn build_recursive(&mut self, node_idx: usize, node_count: &mut usize) {
        let node = &mut self.nodes[node_idx];

        node.bbox = AABB::EMPTY;

        if node.primitive_count <= 2 {
            return;
        }

        for i in 0..node.primitive_count {
            let model = &self.models[self.prim_indices[(node.first_idx + i) as usize]];
            node.bbox = AABB::from((node.bbox, model.bounding_box()));
        }

        /*let mut best_pos = 0.0;
        let mut best_cost = 1e30;
        let mut best_axis = 0;

        for axis in 0..3 {
            for i in 0..self.models.len() {
                let model = &self.models[self.prim_indices[node.first_idx as usize + i]];
                let candidate = model.bounding_box().center().axis(axis);
                let cost = self.evaluate_sah(node_idx, axis, candidate);
                if cost < best_cost {
                    best_pos = candidate;
                    best_axis = axis;
                    best_cost = cost;
                }
            }
        }
        let axis = best_axis;*/
        let axis = node.bbox.longest_axis();

        let start = node.first_idx as usize;
        let end = start + node.primitive_count as usize;

        let sorted = &mut self.prim_indices[start..end];
        sorted.sort_unstable_by(|a, b| {
            let a_center = self.models[*a].bounding_box().center().axis(axis);
            let b_center = self.models[*b].bounding_box().center().axis(axis);

            return a_center.total_cmp(&b_center);
        });

        let left_count = start - node.first_idx as usize;

        // no split possible
        if left_count == 0 || left_count == node.primitive_count as usize {
            return;
        }

        let left_child_idx = *node_count + 0;
        let right_child_idx = *node_count + 1;
        *node_count += 2;

        let node = &self.nodes[node_idx];
        let first_idx = node.first_idx;
        let primitive_count = node.primitive_count;

        // setup split
        self.nodes[left_child_idx].first_idx = first_idx;
        self.nodes[left_child_idx].primitive_count = left_count as u32;
        self.nodes[right_child_idx].first_idx = start as u32;
        self.nodes[right_child_idx].primitive_count = primitive_count - left_count as u32;

        // turn current node into leaf
        self.nodes[node_idx].primitive_count = 0;
        self.nodes[node_idx].first_idx = left_child_idx as u32;

        // build child nodes
        self.build_recursive(left_child_idx, node_count);
        self.build_recursive(right_child_idx, node_count);
    }

    fn evaluate_sah(&self, node_idx: usize, axis: usize, pos: Float) -> Float {
        let node = &self.nodes[node_idx];
        0.0
    }
}
