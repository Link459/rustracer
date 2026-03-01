use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    ray::Ray,
    texture::{Texture, TextureStorage},
    vec3::Vec3,
    Float,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum Skybox {
    #[default]
    Sky,
    Night,
    Hdri(TextureStorage),
}

impl Skybox {
    #[inline(always)]
    pub fn call(&self, ray: &Ray) -> Vec3 {
        match self {
            Skybox::Sky => skybox(ray),
            Skybox::Night => night(ray),
            Skybox::Hdri(ref img) => hdri(ray, img),
        }
    }
}

impl Display for Skybox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Skybox::Sky => write!(f, "background: Sky")?,
            Skybox::Night => write!(f, "background: Night")?,
            Skybox::Hdri(_) => write!(f, "background: HDRI")?,
        }
        return Ok(());
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
    let dir = ray.dir.normalize();
    let u = 0.5 + Float::atan2(dir.x, dir.z) / (2.0 * crate::consts::PI);
    let v = 0.5 + dir.y.asin() / crate::consts::PI;
    return hdri.value(u, v, &dir);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_depth: u32,
    pub skybox: Skybox,
}

impl RenderSettings {
    pub fn new(width: u32, height: u32, samples: u32, max_depth: u32) -> Self {
        Self {
            width,
            height,
            samples,
            max_depth,
            skybox: Skybox::Sky,
        }
    }

    pub fn with_aspect_ratio(
        aspect_ratio: Float,
        width: u32,
        samples: u32,
        max_depth: u32,
    ) -> Self {
        Self {
            width,
            height: (width as Float / aspect_ratio) as u32,
            samples,
            max_depth,
            skybox: Skybox::Sky,
        }
    }
    pub fn with_background(
        width: u32,
        height: u32,
        samples: u32,
        max_depth: u32,
        bg: Skybox,
    ) -> Self {
        Self {
            width,
            height,
            samples,
            max_depth,
            skybox: bg,
        }
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        return Self::with_aspect_ratio(16.0 / 9.0, 400, 100, 50);
    }
}

impl Display for RenderSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " width: {}", self.width)?;
        writeln!(f, " height: {}", self.height)?;
        writeln!(f, " samples: {}", self.samples)?;
        writeln!(f, " max depth: {}", self.max_depth)?;
        writeln!(f, " background: {}", self.skybox)?;
        return Ok(());
    }
}
