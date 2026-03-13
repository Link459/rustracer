use crate::{vec3::Vec3, Float};

pub type Color = Vec3;

pub fn luminance(f: Color) -> Float {
    f.dot(&Vec3::new(0.2125, 0.7154, 0.0721))
}
