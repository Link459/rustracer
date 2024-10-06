use crate::{ray::Ray, vec3::Vec3};
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
pub enum Background {
    Sky,
    Night,
}

impl Into<fn(ray: &Ray) -> Vec3> for Background {
    fn into(self) -> fn(ray: &Ray) -> Vec3 {
        match self {
            Background::Sky => skybox,
            Background::Night => night,
        }
    }
}

pub fn skybox(ray: &Ray) -> Vec3 {
    let unit_dir = ray.dir.normalize();
    let t = 0.5 * (unit_dir.y + 1.0);
    return (1.0 - t) * Vec3::ONE + t * Vec3::new(0.5, 0.7, 1.0);
}

pub fn night(_ray: &Ray) -> Vec3 {
    Vec3::ZERO
}

#[derive(Clone, Copy, Deserialize)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_depth: u32,
    pub background: Background, //pub background: fn(ray: &Ray) -> Vec3,
}

impl RenderConfig {
    pub fn new(width: u32, height: u32, samples: u32, max_depth: u32) -> Self {
        Self {
            width,
            height,
            samples,
            max_depth,
            background: Background::Sky,
        }
    }

    pub fn with_aspect_ratio(aspect_ratio: f64, width: u32, samples: u32, max_depth: u32) -> Self {
        Self {
            width,
            height: (width as f64 / aspect_ratio) as u32,
            samples,
            max_depth,
            background: Background::Sky,
        }
    }
    pub fn with_background(
        width: u32,
        height: u32,
        samples: u32,
        max_depth: u32,
        bg: Background,
    ) -> Self {
        Self {
            width,
            height,
            samples,
            max_depth,
            background: bg,
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        return Self::with_aspect_ratio(16.0 / 9.0, 400, 100, 50);
    }
}
