use std::{cell::Ref, ops::Add};

use crate::{interval::Interval, ray::Ray, vec3::Vec3, Float};

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Clone, Copy, Debug)]
pub struct AABB {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl AABB {
    pub const EMPTY: AABB = AABB {
        x: Interval::EMPTY,
        y: Interval::EMPTY,
        z: Interval::EMPTY,
    };

    pub const UNIVERSE: AABB = AABB {
        x: Interval::UNIVERSE,
        y: Interval::UNIVERSE,
        z: Interval::UNIVERSE,
    };

    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }

    pub fn pad(&self) -> AABB {
        let delta = 0.0001;
        let new_x = match self.x.size() >= delta {
            true => self.x,
            false => self.x.expand(delta),
        };
        let new_y = match self.y.size() >= delta {
            true => self.y,
            false => self.y.expand(delta),
        };
        let new_z = match self.z.size() >= delta {
            true => self.z,
            false => self.z.expand(delta),
        };

        return Self::new(new_x, new_y, new_z);
    }

    pub fn hit(&self, ray: &Ray, mut ray_t: Interval) -> bool {
        let orig = ray.orig;
        let dir = ray.dir;
        for axis in 0..3 {
            let ax = self.axis(axis);
            let adinv = 1.0 / dir[axis];

            let t0 = (ax.min - orig[axis]) * adinv;
            let t1 = (ax.max - orig[axis]) * adinv;

            if t0 < t1 {
                if t0 > ray_t.min {
                    ray_t.min = t0;
                }
                if t1 < ray_t.max {
                    ray_t.max = t1;
                }
            } else {
                if t1 > ray_t.min {
                    ray_t.min = t1;
                }
                if t0 < ray_t.max {
                    ray_t.max = t0;
                }
            }

            if ray_t.max <= ray_t.min {
                return false;
            }
            /*let inv_d = 1.0 / ray.dir[a];
            let orig = ray.orig[a];

            let mut t0 = (self.axis(a).min - orig) * inv_d;
            let mut t1 = (self.axis(a).max - orig) * inv_d;

            if inv_d < 0.0 {
                swap(&mut t0, &mut t1);
            }

            ray_t.min = ray_t.min.max(t0);
            ray_t.max = ray_t.max.min(t1);

            if ray_t.max <= ray_t.min {
                return false;
            }*/
        }
        return true;
    }

    /// extends an AABB by another one
    pub fn surrounding_box(&self, box0: &Self) -> Self {
        let min = Vec3::new(
            Float::min(self.x.min, box0.x.min),
            Float::min(self.y.min, box0.y.min),
            Float::min(self.z.min, box0.z.min),
        );
        let max = Vec3::new(
            Float::max(self.x.max, box0.x.max),
            Float::max(self.y.max, box0.y.max),
            Float::max(self.z.max, box0.z.max),
        );

        return Self::from((min, max));
    }

    pub fn min(&self) -> Vec3 {
        return Vec3::new(self.x.min, self.y.min, self.z.min);
    }

    pub fn max(&self) -> Vec3 {
        return Vec3::new(self.x.max, self.y.max, self.z.max);
    }

    pub fn min_axis(&self, axis: usize) -> Float {
        return self.axis(axis).min;
    }

    pub fn max_axis(&self, axis: usize) -> Float {
        return self.axis(axis).max;
    }

    pub fn axis(&self, n: usize) -> Interval {
        return match n {
            1 => self.y,
            2 => self.z,
            _ => self.x,
        };
    }

    pub fn longest_axis(&self) -> usize {
        if self.x.size() > self.y.size() {
            if self.x.size() > self.z.size() {
                return 0;
            }
            return 2;
        }
        if self.y.size() > self.z.size() {
            return 1;
        }
        return 2;
    }

    pub fn diagonal(&self) -> Vec3 {
        let min = Vec3::new(self.x.min, self.y.min, self.z.min);
        let max = Vec3::new(self.x.max, self.y.max, self.z.max);
        return max - min;
    }

    pub fn half_area(&self) -> Float {
        let d = self.diagonal();
        return (d.x + d.y) * d.z + d.x * d.y;
    }

    pub fn center(&self) -> Vec3 {
        let half_point = |i: Interval| (i.max + i.min).abs() / 2.0;
        return Vec3::new(half_point(self.x), half_point(self.y), half_point(self.z));
    }
}

impl Add<Vec3> for AABB {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        return AABB::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z);
    }
}

impl From<(Vec3, Vec3)> for AABB {
    fn from(value: (Vec3, Vec3)) -> Self {
        let x = Interval::new(
            Float::min(value.0[0], value.1[0]),
            Float::max(value.0[0], value.1[0]),
        );
        let y = Interval::new(
            Float::min(value.0[1], value.1[1]),
            Float::max(value.0[1], value.1[1]),
        );
        let z = Interval::new(
            Float::min(value.0[2], value.1[2]),
            Float::max(value.0[2], value.1[2]),
        );

        return Self::new(x, y, z);
    }
}

impl From<(Self, Self)> for AABB {
    fn from(value: (Self, Self)) -> Self {
        let x = Interval::from((value.0.x, value.1.x));
        let y = Interval::from((value.0.y, value.1.y));
        let z = Interval::from((value.0.z, value.1.z));
        return AABB::new(x, y, z);
    }
}

impl From<Ref<'_, Self>> for AABB {
    fn from(value: Ref<Self>) -> Self {
        *value
    }
}

unsafe impl Send for AABB {}
unsafe impl Sync for AABB {}
