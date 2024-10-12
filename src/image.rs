use std::ptr;

use crate::{present::PresentationEvent, render::RenderConfig, vec3::Vec3};
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use winit::event_loop::EventLoopProxy;

#[derive(Default, Debug, Clone)]
pub struct Image {
    pub buffer: Vec<u8>,
    pub width: u32,
    pub height: u32,
    samples: f64,
}

impl<'de> Deserialize<'de> for Image {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl Serialize for Image {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
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
                let index = self.index(h, w);
                self.write(color, index);
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
                let index = self.index(h, w);
                self.write(color, index);
            })
        })
    }

    #[inline]
    pub fn compute_parallel_present<F>(
        &mut self,
        work_load: F,
        proxy: EventLoopProxy<PresentationEvent>,
    ) -> ()
    where
        F: Fn(u32, u32) -> Vec3 + Send + Sync,
    {
        (0..self.height).into_par_iter().for_each(|h| {
            (0..self.width).into_par_iter().for_each(|w| {
                let color = work_load(w, h);
                let index = self.index(h, w);
                proxy
                    .send_event(PresentationEvent {
                        color: color.clone(),
                        x: w,
                        y: h,
                    })
                    .unwrap();
                self.write(color, index);
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
        }
    }

    pub fn into_image_buffer(self) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        image::RgbImage::from_vec(self.width, self.height, self.buffer.clone()).unwrap()
    }

    pub fn index(&self, row: u32, column: u32) -> usize {
        self.buffer.len() - 3 * (row * self.width + column) as usize
    }

    pub fn iter_pixels<'a>(&'a self) -> ImageIterator<'a> {
        return ImageIterator::new(self);
    }
}

impl From<&RenderConfig> for Image {
    fn from(v: &RenderConfig) -> Self {
        return Self::new(v.width, v.height, v.samples.into());
    }
}

impl From<ImageBuffer<Rgb<u8>, Vec<u8>>> for Image {
    fn from(value: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Self {
        let v = value.to_vec();
        Self {
            buffer: v,
            width: value.width(),
            height: value.height(),
            samples: 0.0,
        }
    }
}

pub struct ImageIterator<'a> {
    image: &'a Image,
    x: usize,
    y: usize,
    idx: usize,
}

impl<'a> ImageIterator<'a> {
    pub fn new(image: &'a Image) -> Self {
        Self {
            image,
            x: 0,
            y: 0,
            idx: 0,
        }
    }
}

impl<'a> Iterator for ImageIterator<'a> {
    type Item = (usize, usize, &'a [u8]);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.image.buffer.len() {
            return None;
        }

        if self.x >= self.image.width as usize {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        let buf = &self.image.buffer[self.idx..(self.idx + 3)];
        self.idx += 3;
        Some((x, y, buf))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.image.buffer.len();
        (len, Some(len))
    }
}
