#![allow(dead_code)]
#![allow(clippy::needless_return)]

pub type Float = f32;
pub mod consts {
    use crate::Float;
    pub const PI: Float = 3.14159265358979323846264338327950288;
    pub const INV_PI: Float = 1.0 / PI;
    pub const TAU: Float = 6.28318530717958647692528676655900577;
    pub const PHI: Float = 1.618033988749894848204586834365638118;
    pub const EGAMMA: Float = 0.577215664901532860606512090082402431;
    pub const FRAC_PI_2: Float = 1.57079632679489661923132169163975144;
    pub const FRAC_PI_3: Float = 1.04719755119659774615421446109316763;
    pub const FRAC_PI_4: Float = 0.785398163397448309615660845819875721;
    pub const FRAC_PI_6: Float = 0.52359877559829887307710723054658381;
    pub const FRAC_PI_8: Float = 0.39269908169872415480783042290993786;
    pub const FRAC_1_PI: Float = 0.318309886183790671537767526745028724;
    pub const FRAC_1_SQRT_PI: Float = 0.564189583547756286948079451560772586;
    pub const FRAC_1_SQRT_2PI: Float = 0.398942280401432677939946059934381868;
    pub const FRAC_2_PI: Float = 0.636619772367581343075535053490057448;
    pub const FRAC_2_SQRT_PI: Float = 1.12837916709551257389615890312154517;
    pub const SQRT_2: Float = 1.41421356237309504880168872420969808;
    pub const FRAC_1_SQRT_2: Float = 0.707106781186547524400844362104849039;
    pub const SQRT_3: Float = 1.732050807568877293527446341505872367;
    pub const FRAC_1_SQRT_3: Float = 0.577350269189625764509148780501957456;
    pub const E: Float = 2.71828182845904523536028747135266250;
}

pub mod color;
pub mod aabb;
pub mod bvh;
pub mod camera;
pub mod denoise;
pub mod hittable;
pub mod image;
pub mod integrator;
pub mod interval;
pub mod light;
pub mod material;
pub mod model;
pub mod moving_sphere;
pub mod onb;
pub mod pdf;
pub mod perlin;
pub mod present;
pub mod random;
pub mod ray;
pub mod render;
pub mod sampler;
pub mod scene;
pub mod settings;
pub mod texture;
pub mod utils;
pub mod vec3;
pub mod world;
pub mod world_options;
