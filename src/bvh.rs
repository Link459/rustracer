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
    pub fn new(models: Vec<Model>, start: usize, end: usize) -> Self {
        let comp: usize = thread_rng().gen_range(0..2);

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
                node = BvhNode {
                    left: models[start].clone(),
                    right: models[start + 1].clone(),
                    bbox: AABB::default(),
                };

                //TODO: sort this by a certain axis
                //and also do the recursive init for left and right
                let to_sort = &models[start..end];

                let mid = start + span / 2;
            }
        }

        node.bbox = AABB::from((
            node.left.bounding_box().to_owned(),
            node.right.bounding_box().to_owned(),
        ));
        return node;
    }
    pub fn from_world(world: World) -> Self {
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
        let right;
        if let Some(hit) = &left {
            let interval = Interval::new(ray_t.min, hit.0.t);
            right = self.right.hit(ray, interval);
        } else {
            right = self.right.hit(ray, ray_t);
        }

        if right.is_some() {
            return right;
        }
        return left;
    }

    fn bounding_box(&self) -> &AABB {
        return &self.bbox;
    }
}
