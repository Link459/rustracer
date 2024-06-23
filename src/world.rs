use crate::interval::Interval;
use crate::ray::Ray;
use crate::{
    aabb::AABB,
    hittable::{HitPayload, Hittable},
};
use crate::{material::Material, model::Model};

#[derive(Clone, Default, Debug)]
pub struct World {
    pub entities: Vec<Model>,
    bbox: AABB,
}

impl World {
    pub fn new() -> Self {
        return Self {
            entities: Vec::new(),
            bbox: AABB::default(),
        };
    }

    pub fn clear(&mut self) -> () {
        self.entities.clear();
    }

    pub fn add(&mut self, entity: Model) -> () {
        self.bbox = AABB::from((self.bbox, *entity.bounding_box()));
        self.entities.push(entity);
    }

    pub fn add_slice(&mut self, models: impl IntoIterator<Item = Model>) -> () {
        self.entities.extend(models);
    }

    pub fn extend(&mut self, other: World) -> () {
        self.entities.extend(other.entities);
    }
}

impl Hittable for World {
    #[inline]
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, Material)> {
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

    fn bounding_box(&self) -> &AABB {
        return &self.bbox;
        /*if self.entities.is_empty() {
            return None;
        };

        let mut first_box = true;
        let mut output_box = AABB::new(Vec3::ZERO, Vec3::ZERO);

        for object in self.entities.iter() {
            if let Some(b) = object.bounding_box(time0, time1) {
                output_box = match first_box {
                    true => b,
                    false => output_box.surrounding_box(&b),
                };
                first_box = false;
            } else {
                return None;
            }
        }

        return Some(output_box);*/
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
