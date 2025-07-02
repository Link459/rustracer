use std::{fmt, write};

use crate::vec3::Vec3;

#[derive(Debug)]
pub struct Ray {
    pub orig: Vec3,
    pub dir: Vec3,
    pub time: f64,
}

impl Ray {
    pub fn new(orig: Vec3, dir: Vec3, time: f64) -> Self {
        return Self { orig, dir, time };
    }

    pub fn at(&self, t: f64) -> Vec3 {
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
