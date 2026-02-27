use rand::{ RngExt};
use serde::{Deserialize, Serialize};

use crate::interval::Interval;
use crate::material::{MaterialId};
use crate::model::Model;
use crate::ray::Ray;
use crate::vec3::Vec3;
use crate::Float;
use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct World {
    pub entities: Vec<Model>,
    #[serde(skip)]
    bbox: AABB,
}

impl World {
    pub fn new() -> Self {
        return Self {
            entities: Vec::new(),
            bbox: AABB::default(),
        };
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn add<T>(&mut self, entity: T)
    where
        T: Hittable + Into<Model>,
    {
        self.bbox = AABB::from((self.bbox, entity.bounding_box()));
        self.entities.push(Into::into(entity));
    }

    pub fn add_slice(&mut self, models: impl IntoIterator<Item = Model>) {
        self.entities.extend(models);
    }

    pub fn extend(&mut self, other: World) {
        self.entities.extend(other.entities);
    }
}

impl Hittable for World {
    #[inline]
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialId)> {
        let mut closest_so_far = ray_t.max;
        let mut res = None;

        for e in self.entities.iter() {
            let new_ray_t = Interval::new(ray_t.min, closest_so_far);
            if let Some((payload, material)) = e.hit(ray, new_ray_t) {
                closest_so_far = payload.t;
                res = Some((payload, material));
            }
        }
        return res;
    }

    fn bounding_box(&self) -> AABB {
        return self.bbox;
    }

    //TODO: handle a light count of 0
    fn pdf_value(&self, origin: &Vec3, dir: &Vec3) -> Float {
        let size = self.entities.len();

        let weight = 1.0 / size as Float;

        let mut sum = 0.0;

        for entity in &self.entities {
            sum += weight * entity.pdf_value(origin, dir);
        }

        return sum;
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let size = self.entities.len();

        let idx = rand::rng().random_range(0..size);
        return self.entities[idx].random(origin);
    }
}

impl From<Vec<Model>> for World {
    fn from(value: Vec<Model>) -> Self {
        Self {
            entities: value,
            bbox: AABB::default(),
        }
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}
