use std::ops::Add;

#[derive(Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

pub const EMPTY: Interval = Interval {
    min: f64::INFINITY,
    max: -f64::INFINITY,
};
pub const UNIVERSE: Interval = Interval {
    min: -f64::INFINITY,
    max: f64::INFINITY,
};

impl Interval {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    // add code here

    pub fn contains(&self, x: f64) -> bool {
        return self.min <= x && x <= self.max;
    }

    pub fn surrounds(&self, x: f64) -> bool {
        return self.min < x && x < self.max;
    }
    pub fn size(&self) -> f64 {
        return self.max - self.min;
    }

    pub fn expand(self, delta: f64) -> Self {
        let padding = delta / 2.0;
        return Self::new(self.min - padding, self.max + padding);
    }

    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            return self.min;
        };
        if x > self.max {
            return self.max;
        };
        return x;
    }
}

impl Add<f64> for Interval {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        return Interval::new(self.min + rhs, self.max + rhs);
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self {
            min: (f64::INFINITY),
            max: (-f64::INFINITY),
        }
    }
}

impl From<(Interval, Interval)> for Interval {
    fn from(value: (Interval, Interval)) -> Self {
        let min = f64::min(value.0.min, value.1.min);
        let max = f64::max(value.0.max, value.1.max);
        Self { min, max }
    }
}
