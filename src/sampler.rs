use rand::{rngs::SmallRng, Rng};

use crate::Float;

pub trait Sampler {
    fn sample_pixel(&mut self, x: Float, y: Float, sample_idx: u32) -> [Float; 2];
    fn sample_1d(&mut self) -> Float;
    fn sample_2d(&mut self) -> [Float; 2];
}

#[derive(Clone)]
pub struct IndependentSampler {
    rng: SmallRng,
}

impl IndependentSampler {
    pub fn new(rng: SmallRng) -> Self {
        return Self { rng };
    }
}

impl Sampler for IndependentSampler {
    fn sample_pixel(&mut self, _x: Float, _y: Float, _sample_idx: u32) -> [Float; 2] {
        return self.sample_2d();
    }

    fn sample_1d(&mut self) -> Float {
        return self.rng.random_range(0.0..1.0);
    }

    fn sample_2d(&mut self) -> [Float; 2] {
        return [self.sample_1d(), self.sample_1d()];
    }
}
