use crate::{perlin::Perlin, vec3::Vec3};
use image::{open, GenericImageView};

#[derive(Clone, Debug)]
pub enum Texture {
    SolidColor(SolidColor),
    Chess(ChessTexture),
    Noise(NoiseTexture),
    Image(ImageTexture),
}

impl TextureValue for Texture {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        match self {
            Texture::SolidColor(ref t) => t.value(u, v, p),
            Texture::Chess(ref t) => t.value(u, v, p),
            Texture::Noise(ref t) => t.value(u, v, p),
            Texture::Image(ref t) => t.value(u, v, p),
        }
    }
}

pub trait TextureValue {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3;
}

#[derive(Clone, Debug, Copy)]
pub struct SolidColor {
    pub color_value: Vec3,
}

impl SolidColor {
    pub fn new(color_value: Vec3) -> Texture {
        return Texture::SolidColor(SolidColor { color_value });
    }
}

impl TextureValue for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Vec3) -> Vec3 {
        self.color_value
    }
}

#[derive(Clone, Debug)]
pub struct ChessTexture {
    pub odd: Box<Texture>,
    pub even: Box<Texture>,
}

impl ChessTexture {
    pub fn new(odd: Box<Texture>, even: Box<Texture>) -> Texture {
        return Texture::Chess(Self { odd, even });
    }
}

impl TextureValue for ChessTexture {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        let sines = f64::sin(10.0 * p.x) * f64::sin(10.0 * p.y) * f64::sin(10.0 * p.z);
        if sines < 0.0 {
            return self.odd.value(u, v, p);
        } else {
            return self.even.value(u, v, p);
        }
    }
}

#[derive(Clone, Debug)]
pub struct NoiseTexture {
    perlin: Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Texture {
        return Texture::Noise(Self {
            perlin: Perlin::new(),
            scale,
        });
    }
}

impl TextureValue for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Vec3) -> Vec3 {
        return Vec3::ONE * 0.5 * (1.0 + (self.scale * p.x + 10.0 * self.perlin.turb(&p, 7)).sin());
    }
}

#[derive(Clone, Debug)]
pub struct ImageTexture {
    buffer: Vec<u8>,
    nx: u32,
    ny: u32,
}

impl ImageTexture {
    pub fn new(file_path: &str) -> Texture {
        let buffer =
            open(file_path).expect(format!("failed to open image with path: {file_path}").as_str());
        let (nx, ny) = buffer.dimensions();

        return Texture::Image(Self {
            nx,
            ny,
            buffer: buffer.into_bytes(),
        });
    }
}

impl TextureValue for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: &Vec3) -> Vec3 {
        let nx = self.nx as usize;
        let ny = self.ny as usize;
        let mut i = (u * nx as f64) as usize;
        let mut j = ((1.0 - v) * ny as f64) as usize;
        if i > nx - 1 {
            i = nx - 1
        }
        if j > ny - 1 {
            j = ny - 1
        }

        let index = 3 * i + ((3 * nx) * j);
        let r = self.buffer[index] as f64 / 255.0;
        let g = self.buffer[index + 1] as f64 / 255.0;
        let b = self.buffer[index + 2] as f64 / 255.0;
        Vec3::new(r, g, b)
    }
}
