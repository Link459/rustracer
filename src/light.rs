use crate::{vec3::Vec3, Float};

pub trait Light {
    fn l(p: Vec3,n: Vec3,uv: [Float; 2], w: Vec3) -> Vec3 {
        return Vec3::ZERO;
    }
}

struct LightSample {
    l: Vec3,
    wo: Vec3,
    pdf: Float,
}
