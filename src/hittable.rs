use std::{
    fmt::{self, Display, Formatter},
    write,
};


use crate::{
    aabb::AABB, interval::Interval, material::MaterialStorage, ray::Ray, vec3::Vec3,
};

#[derive(Default, Debug)]
pub struct HitPayload {
    ///The position where the hit occured
    pub p: Vec3,
    /// the surface normal
    pub normal: Vec3,
    pub t: f64,
    /// The interpolated u coordinate between [0,1] (used for texture sampling)
    pub u: f64,
    /// The interpolated v coordinate between [0,1] (used for texture sampling)
    pub v: f64,
    pub front_face: bool,
}

impl HitPayload {
    pub fn new(p: Vec3, normal: Vec3, t: f64, u: f64, v: f64) -> Self {
        Self {
            p,
            normal,
            t,
            u,
            v,
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = Vec3::dot(&ray.dir, &outward_normal) < 0.0;
        self.normal = match self.front_face {
            true => outward_normal,
            false => -outward_normal,
        }
    }
}

impl Display for HitPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.p)?;
        write!(f, "{:?}", self.normal)?;
        write!(f, "{:?}", self.t)?;
        write!(f, "{:?}", self.u)?;
        write!(f, "{:?}", self.v)?;
        write!(f, "{:?}", self.front_face)?;
        Ok(())
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)>;
    fn bounding_box(&self) -> AABB;
    fn pdf_value(&self, _origin: &Vec3, _dir: &Vec3) -> f64 {
        return 0.0;
    }
    fn random(&self, _origin: &Vec3) -> Vec3 {
        return Vec3::new(1.0, 0.0, 0.0);
    }
}
