use crate::{ray::Ray, vec3::Vec3};

pub fn skybox(ray: &Ray) -> Vec3 {
    let unit_dir = ray.dir.normalize();
    let t = 0.5 * (unit_dir.y + 1.0);
    return (1.0 - t) * Vec3::ONE + t * Vec3::new(0.5, 0.7, 1.0);
}

#[derive(Clone, Copy)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_depht: u32,
    pub background: fn(ray: &Ray) -> Vec3,
}

impl RenderConfig {
    pub fn new(width: u32, height: u32, samples: u32, max_depht: u32) -> Self {
        Self {
            width,
            height,
            samples,
            max_depht,
            background: skybox,
        }
    }

    pub fn with_aspect_ratio(aspect_ratio: f64, width: u32, samples: u32, max_depht: u32) -> Self {
        Self {
            width,
            height: (width as f64 / aspect_ratio) as u32,
            samples,
            max_depht,
            background: skybox,
        }
    }
    pub fn with_background(
        width: u32,
        height: u32,
        samples: u32,
        max_depht: u32,
        bg: fn(&Ray) -> Vec3,
    ) -> Self {
        Self {
            width,
            height,
            samples,
            max_depht,
            background: bg,
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        return Self::with_aspect_ratio(16.0 / 9.0, 400, 100, 50);
    }
}
