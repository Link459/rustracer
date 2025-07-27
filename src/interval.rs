use std::{fmt::Display, ops::Add};

use serde::{Deserialize, Serialize};

use crate::Float;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Interval {
    pub min: Float,
    pub max: Float,
}

impl Interval {
    pub const EMPTY: Interval = Interval {
        min: Float::INFINITY,
        max: -Float::INFINITY,
    };
    pub const UNIVERSE: Interval = Interval {
        min: -Float::INFINITY,
        max: Float::INFINITY,
    };

    pub fn new(min: Float, max: Float) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, x: Float) -> bool {
        return self.min <= x && x <= self.max;
    }

    pub fn surrounds(&self, x: Float) -> bool {
        return self.min < x && x < self.max;
    }
    pub fn size(&self) -> Float {
        return self.max - self.min;
    }

    pub fn expand(self, delta: Float) -> Self {
        let padding = delta / 2.0;
        return Self::new(self.min - padding, self.max + padding);
    }

    pub fn clamp(&self, x: Float) -> Float {
        if x < self.min {
            return self.min;
        };
        if x > self.max {
            return self.max;
        };
        return x;
    }
}

impl Add<Float> for Interval {
    type Output = Self;

    fn add(self, rhs: Float) -> Self::Output {
        return Interval::new(self.min + rhs, self.max + rhs);
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self {
            min: (Float::INFINITY),
            max: (-Float::INFINITY),
        }
    }
}

impl From<(Interval, Interval)> for Interval {
    fn from(value: (Interval, Interval)) -> Self {
        let min = Float::min(value.0.min, value.1.min);
        let max = Float::max(value.0.max, value.1.max);
        Self { min, max }
    }
}

impl Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "min: {}", self.min)?;
        write!(f, "max: {}", self.max)?;
        return Ok(());
    }
}
