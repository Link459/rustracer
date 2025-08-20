use std::{fmt, write};

use crate::{vec3::Vec3, Float};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub orig: Vec3,
    pub dir: Vec3,
    pub time: Float,
}

impl Ray {
    pub fn new(orig: Vec3, dir: Vec3, time: Float) -> Self {
        return Self { orig, dir, time };
    }

    pub fn new_ray_to(from: Vec3,  to: Vec3, time: Float) -> Self {
        let d = to - from;
        return Self::new(from, d, time);
    }

    pub fn at(&self, t: Float) -> Vec3 {
        return self.orig + t * self.dir;
    }
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.orig)?;
        write!(f, "{:?}", self.dir)?;

        return Ok(());
    }
}
