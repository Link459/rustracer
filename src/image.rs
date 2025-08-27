use std::{path::Path, ptr};

use crate::{
    present::PresentationEvent, render::RenderSettings, settings::Settings, vec3::Vec3, Float,
};
use image::{ImageBuffer, Rgb};
use rayon::prelude::*;
use winit::event_loop::EventLoopProxy;

#[derive(Default, Debug, Clone)]
pub struct Image {
    pub buffer: Vec<Float>,
    pub width: u32,
    pub height: u32,
}

pub type ImageBufferGlue<T = Float> = image::ImageBuffer<image::Rgb<T>, Vec<T>>;

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        let len = 3 * (width * height) as usize;
        let mut img = Self {
            buffer: Vec::with_capacity(len),
            width,
            height,
        };
        unsafe { img.buffer.set_len(len) };

        img.buffer.fill(0.0);
        return img;
    }

    #[inline(always)]
    pub fn compute<F>(&mut self, work_load: F)
    where
        F: Fn(u32, u32) -> Vec3,
    {
        (0..self.height).for_each(|h| {
            (0..self.width).for_each(|w| {
                let color = work_load(w, h);
                let index = self.index(h, w);
                self.write(color, index);
            })
        })
    }

    #[inline]
    pub fn compute_parallel<F>(&mut self, work_load: F)
    where
        F: Fn(u32, u32) -> Vec3 + Send + Sync,
    {
        (0..self.height).into_par_iter().for_each(|h| {
            (0..self.width).into_par_iter().rev().for_each(|w| {
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
    ) where
        F: Fn(u32, u32) -> Vec3 + Send + Sync,
    {
        (0..self.height).into_par_iter().for_each(|h| {
            (0..self.width).into_par_iter().rev().for_each(|w| {
                let color = work_load(w, h);
                let index = self.index(h, w);
                //let index = self.index(w, h);

                proxy
                    .send_event(PresentationEvent { color, x: w, y: h })
                    .unwrap();
                self.write(color, index);
            })
        })
    }

    #[inline(always)]
    pub fn write(&self, color: Vec3, index: usize) {
        let ptr = self.buffer.as_ptr() as *mut Float;
        unsafe {
            ptr::write(ptr.add(index), color.x);
            ptr::write(ptr.add(index + 1), color.y);
            ptr::write(ptr.add(index + 2), color.z);
        }
    }

    #[inline(always)]
    pub fn read(&self, index: usize) -> Vec3 {
        let ptr = self.buffer.as_ptr() as *mut Float;
        let mut color = Vec3::ZERO;
        unsafe {
            color.x = ptr::read(ptr.add(index));
            color.y = ptr::read(ptr.add(index + 1));
            color.z = ptr::read(ptr.add(index + 2));
        }
        return color;
    }

    pub fn read_ptr(ptr: *mut Float, index: usize) -> Vec3 {
        let mut color = Vec3::ZERO;
        unsafe {
            color.x = ptr::read(ptr.add(index));
            color.y = ptr::read(ptr.add(index + 1));
            color.z = ptr::read(ptr.add(index + 2));
        }
        return color;
    }

    pub fn into_image_buffer(self) -> ImageBufferGlue {
        return ImageBufferGlue::from_vec(self.width, self.height, self.buffer.clone()).unwrap();
    }

    pub fn into_bytes(self) -> ImageBufferGlue<u8> {
        let mut buf = Vec::with_capacity(self.buffer.capacity());

        for chunck in self.buffer.chunks(3) {
            let r = Float::sqrt(chunck[0]);
            let g = Float::sqrt(chunck[1]);
            let b = Float::sqrt(chunck[2]);

            buf.push((256.0 * r.clamp(0.0, 0.999)) as u8);
            buf.push((256.0 * g.clamp(0.0, 0.999)) as u8);
            buf.push((256.0 * b.clamp(0.0, 0.999)) as u8);
        }

        return ImageBufferGlue::from_vec(self.width, self.height, buf).unwrap();
    }

    pub fn save(
        self,
        settings: &Settings,
        path: impl AsRef<Path>,
    ) -> Result<(), image::ImageError> {
        let buf = self.into_bytes();

        let mut new_path = settings.output.clone();
        new_path.push(path);

        buf.save(new_path).unwrap();
        return Ok(());
    }

    pub fn index(&self, row: u32, column: u32) -> usize {
        let column = self.width - column;

        self.buffer.len() - 3 * (row * self.width + column) as usize
    }

    pub fn iter_pixels(&self) -> ImageIterator<'_> {
        return ImageIterator::new(self);
    }
}

impl From<&RenderSettings> for Image {
    fn from(v: &RenderSettings) -> Self {
        return Self::new(v.width, v.height);
    }
}

impl From<ImageBuffer<Rgb<Float>, Vec<Float>>> for Image {
    fn from(value: ImageBuffer<Rgb<Float>, Vec<Float>>) -> Self {
        let v = value.to_vec();
        Self {
            buffer: v,
            width: value.width(),
            height: value.height(),
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
    type Item = (usize, usize, &'a [Float]);

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
