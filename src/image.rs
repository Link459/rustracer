use crate::{render::RenderConfig, vec3::Vec3};
use rayon::prelude::*;

pub struct Image {
    pub buffer: Vec<u8>,
    width: u32,
    height: u32,
    samples: f64,
}

impl Image {
    pub fn new(width: u32, height: u32, samples: f64) -> Self {
        Self {
            buffer: Vec::with_capacity((width * height) as usize),
            width,
            height,
            samples,
        }
    }

    #[inline]
    pub fn compute<F>(&mut self, work_load: F) -> ()
    where
        F: Fn(u32, u32) -> Vec3 + Copy,
    {
        let p = (0..self.height)
            .into_iter()
            .rev()
            .map(|h| (0..self.width).into_iter().map(move |w| work_load(h, w)))
            .flatten()
            .collect::<Vec<Vec3>>();
        for i in p {
            self.write(i);
        }
    }

    #[inline]
    pub fn compute_parallel<F>(&mut self, work_load: F) -> ()
    where
        F: Fn(u32, u32) -> Vec3 + Send + Sync,
    {
        let p = (0..self.height)
            .into_par_iter()
            .rev()
            .map(|h| {
                (0..self.width)
                    .into_par_iter()
                    .map(|w| work_load(w, h))
                    .collect::<Vec<Vec3>>()
            })
            .flatten()
            .collect::<Vec<Vec3>>();
        for i in p {
            self.write(i);
        }
    }

    #[inline]
    pub fn write(&mut self, add_color: Vec3) -> () {
        let mut r = add_color.x;
        let mut g = add_color.y;
        let mut b = add_color.z;

        let scale = 1.0 / self.samples;
        r = f64::sqrt(scale * r);
        g = f64::sqrt(scale * g);
        b = f64::sqrt(scale * b);

        self.buffer.push((256.0 * r.clamp(0.0, 0.999)) as u8);
        self.buffer.push((256.0 * g.clamp(0.0, 0.999)) as u8);
        self.buffer.push((256.0 * b.clamp(0.0, 0.999)) as u8);
    }
}

impl From<RenderConfig> for Image {
    fn from(value: RenderConfig) -> Self {
        Self {
            buffer: Vec::with_capacity((value.width * value.height) as usize),
            width: value.width,
            height: value.height,
            samples: value.samples as f64,
        }
    }
}
