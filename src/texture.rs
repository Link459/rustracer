use std::fmt;

use crate::{image::Image, perlin::Perlin, vec3::Vec3};
use image::{open, GenericImageView};
use serde::{
    de::{self, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TextureStorage {
    SolidColor(SolidColor),
    Chess(ChessTexture),
    Noise(NoiseTexture),
    Image(ImageTexture),
}

impl Texture for TextureStorage {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        match self {
            TextureStorage::SolidColor(ref t) => t.value(u, v, p),
            TextureStorage::Chess(ref t) => t.value(u, v, p),
            TextureStorage::Noise(ref t) => t.value(u, v, p),
            TextureStorage::Image(ref t) => t.value(u, v, p),
        }
    }
}

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3;
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct SolidColor {
    pub color_value: Vec3,
}

impl SolidColor {
    pub fn new(color_value: Vec3) -> TextureStorage {
        return TextureStorage::SolidColor(SolidColor { color_value });
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Vec3) -> Vec3 {
        self.color_value
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChessTexture {
    pub odd: Box<TextureStorage>,
    pub even: Box<TextureStorage>,
}

impl ChessTexture {
    pub fn new(odd: Box<TextureStorage>, even: Box<TextureStorage>) -> TextureStorage {
        return TextureStorage::Chess(Self { odd, even });
    }
}

impl Texture for ChessTexture {
    fn value(&self, u: f64, v: f64, p: &Vec3) -> Vec3 {
        let sines = f64::sin(10.0 * p.x) * f64::sin(10.0 * p.y) * f64::sin(10.0 * p.z);
        if sines < 0.0 {
            return self.odd.value(u, v, p);
        } else {
            return self.even.value(u, v, p);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NoiseTexture {
    #[serde(skip)]
    perlin: Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> TextureStorage {
        return TextureStorage::Noise(Self {
            perlin: Perlin::new(),
            scale,
        });
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Vec3) -> Vec3 {
        return Vec3::ONE * 0.5 * (1.0 + (self.scale * p.x + 10.0 * self.perlin.turb(&p, 7)).sin());
    }
}

#[derive(Clone, Debug)]
pub struct ImageTexture {
    buffer: Vec<u8>,
    nx: u32,
    ny: u32,
    path: String,
}

impl ImageTexture {
    pub fn new(file_path: &str) -> TextureStorage {
        let buffer =
            open(file_path).expect(format!("failed to open image with path: {file_path}").as_str());
        let (nx, ny) = buffer.dimensions();

        return TextureStorage::Image(Self {
            nx,
            ny,
            buffer: buffer.into_bytes(),
            path: String::from(file_path),
        });
    }

    pub fn from_path(file_path: &str) -> Self {
        let buffer =
            open(file_path).expect(format!("failed to open image with path: {file_path}").as_str());
        let (nx, ny) = buffer.dimensions();

        return Self {
            nx,
            ny,
            buffer: buffer.into_bytes(),
            path: String::from(file_path),
        };
    }
}

impl From<Image> for ImageTexture {
    fn from(value: Image) -> Self {
        Self {
            buffer: value.buffer,
            nx: value.width,
            ny: value.height,
            path: String::default(),
        }
    }
}

impl Texture for ImageTexture {
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

impl Serialize for ImageTexture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.path.is_empty() {
            return serializer.serialize_unit();
        }

        let mut image_texture = serializer.serialize_struct("ImageTexture", 1)?;
        image_texture.serialize_field("path", &self.path)?;
        return image_texture.end();
    }
}

impl<'de> Deserialize<'de> for ImageTexture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            Path,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;
                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`path`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        dbg!(value);
                        match value {
                            "path" => Ok(Field::Path),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ImageTextureVisitor;

        impl<'de> Visitor<'de> for ImageTextureVisitor {
            type Value = ImageTexture;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                return write!(formatter, "path");
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let path = seq
                    .next_element()?
                    .ok_or(de::Error::missing_field("missing field path"))?;
                return Ok(ImageTexture::from_path(path));
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut path = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Path => path = Some(map.next_value()?),
                    }
                }
                //.ok_or(de::Error::missing_field("missing field path"))?;
                dbg!(path);
                let path = path.ok_or_else(|| de::Error::missing_field("path"))?;
                return Ok(ImageTexture::from_path(path));
            }
        }

        const FIELDS: &[&str] = &["path"];
        deserializer.deserialize_struct("ImageTexture", FIELDS, ImageTextureVisitor {})
    }
}
