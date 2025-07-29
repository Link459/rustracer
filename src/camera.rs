use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::{interval::Interval, render::RenderConfig, vec3::Vec3, Float};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub lookfrom: Vec3,
    pub lookat: Vec3,
    pub vup: Vec3,
    pub vfov: Float,
    pub aspect_ratio: Float,
    pub aperture: Float,
    pub focus_dist: Float,
    pub time: Interval,
}

impl Default for CameraConfig {
    fn default() -> Self {
        let lookfrom = Vec3::new(13.0, 2.0, 3.0);
        let lookat = Vec3::new(0.0, 0.0, 0.0);
        let vup = Vec3::new(0.0, 1.0, 0.0);
        let focus_dist = 10.0;
        let aperture = 0.0;

        return Self {
            lookfrom,
            lookat,
            vup,
            vfov: 20.0,
            aspect_ratio: 16.0 / 9.0,
            aperture,
            focus_dist,
            time: Interval::new(0.0, 1.0),
        };
    }
}

impl Display for CameraConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " looking from: {}", self.lookfrom)?;
        writeln!(f, " looking at: {}", self.lookat)?;
        writeln!(f, " fov: {}", self.vfov)?;
        writeln!(f, " aperture: {}", self.aperture)?;
        writeln!(f, " focus distance: {}", self.focus_dist)?;
        writeln!(f, " time: {}", self.time)?;
        return Ok(());
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Camera {
    pub origin: Vec3,
    pub lower_left_corner: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub cu: Vec3,
    pub cv: Vec3,
    pub lens_radius: Float,
    pub time: Interval,
    pub sqrt_samples: Float,
    pub recip_sqrt_samples: Float,
    pub pixel_delta_u: Vec3,
    pub pixel_delta_v: Vec3,
}

impl Camera {
    pub fn new(config: CameraConfig, render_config: &RenderConfig) -> Self {
        return Self::from_camera_config(config, render_config);
    }

    pub fn from_camera_config(config: CameraConfig, render_config: &RenderConfig) -> Self {
        // Vertical field-of-view in degrees
        let theta = crate::consts::PI / 180.0 * config.vfov;
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = config.aspect_ratio * viewport_height;

        let cw = (config.lookfrom - config.lookat).normalize();
        let cu = config.vup.cross(&cw).normalize();
        let cv = cw.cross(&cu);
        let h = config.focus_dist * viewport_width * cu;
        let v = config.focus_dist * viewport_height * cv;

        let llc = config.lookfrom - h / 2.0 - v / 2.0 - config.focus_dist * cw;

        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);
        let pixel_delta_u = viewport_u / render_config.width as Float;
        let pixel_delta_v = viewport_v / render_config.height as Float;

        let sqrt_samples = (render_config.samples as Float).sqrt();
        let recip_sqrt_samples = 1.0 / sqrt_samples;

        let lens_radius = config.aperture / 2.0;

        return Camera {
            origin: config.lookfrom,
            horizontal: h,
            vertical: v,
            lower_left_corner: llc,
            cu,
            cv,
            lens_radius,
            time: config.time,
            sqrt_samples,
            recip_sqrt_samples,
            pixel_delta_u,
            pixel_delta_v,
        };
    }

    pub fn sample_square(&self) -> Vec3 {
        let mut rng = rand::rng();
        return Vec3::new(
            rng.random_range(-0.5..0.5),
            rng.random_range(-0.5..0.5),
            0.0,
        );
    }

    pub fn sample_square_stratified(&self, s_i: Float, s_j: Float) -> Vec3 {
        let mut rng = rand::rng();
        let px = ((s_i + rng.random_range(0.0..1.0)) * self.recip_sqrt_samples) - 0.5;
        let py = ((s_j + rng.random_range(0.0..1.0)) * self.recip_sqrt_samples) - 0.5;
        return Vec3::new(px, py, 0.0);
    }
}

impl Default for Camera {
    fn default() -> Self {
        let config = CameraConfig::default();

        return Self::new(config, &RenderConfig::default());
    }
}

unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}

#[inline(always)]
pub fn random_in_unit_disk() -> Vec3 {
    let mut rng = rand::rng();
    loop {
        let p = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            0.0,
        );
        if p.length_squared() < 1.0 {
            continue;
        }
        return p;
    }
}
