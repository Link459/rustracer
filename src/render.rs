use core::f64;

use serde::{Deserialize, Serialize};

use crate::{
    ray::Ray,
    texture::{Texture, TextureStorage},
    vec3::Vec3,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum Background {
    Sky,
    Night,
    Hdri(TextureStorage),
}

impl Background {
    #[inline(always)]
    pub fn call(&self, ray: &Ray) -> Vec3 {
        match self {
            Background::Sky => skybox(ray),
            Background::Night => night(ray),
            Background::Hdri(ref img) => hdri(ray, img),
        }
    }
}

#[inline(always)]
pub fn skybox(ray: &Ray) -> Vec3 {
    let unit_dir = ray.dir.normalize();
    let t = 0.5 * (unit_dir.y + 1.0);
    return (1.0 - t) * Vec3::ONE + t * Vec3::new(0.5, 0.7, 1.0);
}

#[inline(always)]
pub fn night(_ray: &Ray) -> Vec3 {
    Vec3::ZERO
}

#[inline(always)]
pub fn hdri(ray: &Ray, hdri: &TextureStorage) -> Vec3 {
    let u = 0.5 + f64::atan2(ray.dir.x, ray.dir.z) / (2.0 * f64::consts::PI);
    let v = 0.5 + ray.dir.y.asin() / f64::consts::PI;
    return hdri.value(u, v, &ray.dir);
}

#[derive(Clone, Serialize, Deserialize)]
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
