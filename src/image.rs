use std::ptr;

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
        let len = 3 * (width * height) as usize;
        let mut img = Self {
            buffer: Vec::with_capacity(len),
            width,
            height,
            samples,
        };
        unsafe { img.buffer.set_len(len) };
        return img;
    }

    #[inline]
    pub fn compute<F>(&mut self, work_load: F) -> ()
    where
        F: Fn(u32, u32) -> Vec3,
    {
        (0..self.height).into_iter().for_each(|h| {
            (0..self.width).into_iter().for_each(|w| {
                let color = work_load(w, h);
                let index = self.buffer.len() - (3 * h * self.width + w * 3) as usize;
                self.write(color, index as usize);
            })
        })
    }

    #[inline]
    pub fn compute_parallel<F>(&mut self, work_load: F) -> ()
    where
        F: Fn(u32, u32) -> Vec3 + Send + Sync,
    {
        (0..self.height).into_par_iter().for_each(|h| {
            (0..self.width).into_par_iter().for_each(|w| {
                let color = work_load(w, h);
                let index = self.buffer.len() - (3 * h * self.width + w * 3) as usize;
                self.write(color, index as usize);
            })
        })
    }

    //BUG: fails to write to the buffer
    #[inline]
    pub fn write(&self, add_color: Vec3, index: usize) -> () {
        let mut r = add_color.x;
        let mut g = add_color.y;
        let mut b = add_color.z;

        let scale = 1.0 / self.samples;
        r = f64::sqrt(scale * r);
        g = f64::sqrt(scale * g);
        b = f64::sqrt(scale * b);

        let ptr = self.buffer.as_ptr() as *mut u8;
        unsafe {
            ptr::write(ptr.add(index), (256.0 * r.clamp(0.0, 0.999)) as u8);
            ptr::write(ptr.add(index + 1), (256.0 * g.clamp(0.0, 0.999)) as u8);
            ptr::write(ptr.add(index + 2), (256.0 * b.clamp(0.0, 0.999)) as u8);
            /*self.buffer.push((256.0 * r.clamp(0.0, 0.999)) as u8);
            self.buffer.push((256.0 * g.clamp(0.0, 0.999)) as u8);
            self.buffer.push((256.0 * b.clamp(0.0, 0.999)) as u8);*/
        }
    }
}

impl From<RenderConfig> for Image {
    fn from(v: RenderConfig) -> Self {
        return Self::new(v.width, v.height, v.samples.into());
    }
}
