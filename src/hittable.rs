use std::{
    fmt::{self, Display, Formatter},
    write,
};

use crate::{aabb::AABB, interval::Interval, material::MaterialId, ray::Ray, vec3::Vec3, Float};

#[derive(Default, Debug)]
pub struct HitPayload {
    ///The position where the hit occured
    pub p: Vec3,
    /// the surface normal
    pub normal: Vec3,
    pub t: Float,
    /// The interpolated u coordinate between [0,1] (used for texture sampling)
    pub u: Float,
    /// The interpolated v coordinate between [0,1] (used for texture sampling)
    pub v: Float,
    pub front_face: bool,
}

impl HitPayload {
    pub fn new(p: Vec3, normal: Vec3, t: Float, u: Float, v: Float) -> Self {
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

pub struct HitSample {
    pub p: Vec3,
    pub pdf: Float,
}

pub struct HitSampleContext {
    pub origin: Vec3,
}

pub trait Hittable: Send + Sync {
    //fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialStorage)>;
    fn hit(&self, ray: &Ray, ray_t: Interval) -> Option<(HitPayload, MaterialId)>;
    fn bounding_box(&self) -> AABB;
    fn sample(&self, ctx: &HitSampleContext) -> Option<HitSample> {
        let p = self.random(&ctx.origin);
        let pdf = self.pdf_value(&ctx.origin, &p);

        return Some(HitSample { p, pdf });
    }

    fn pdf_value(&self, _origin: &Vec3, _dir: &Vec3) -> Float {
        return 0.0;
    }
    fn random(&self, _origin: &Vec3) -> Vec3 {
        return Vec3::new(1.0, 0.0, 0.0);
    }
}

pub trait HittableExt: Hittable {
    fn unoccluded(&self, from: Vec3, to: Vec3) -> bool {
        let epsilon = 0.0001;

        let dir = to - from;
        let dist = dir.length();
        let dir = dir.normalize();

        //let ray = Ray::new(from + dir, dir, 0.0);
        let ray = Ray::new(from, dir, 0.0);

        //let interval = Interval::new(epsilon, 1.0 - epsilon);
        let interval = Interval::new(epsilon, dist);

        let hit = self.hit(&ray, interval);
        return hit.is_none();
    }
}
impl<T> HittableExt for T where T: Hittable {}
