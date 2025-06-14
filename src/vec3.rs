use core::panic;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Range, Sub, SubAssign,
};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Vec3 = Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    #[inline(always)]
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    #[inline(always)]
    pub fn normalize(self) -> Vec3 {
        self / self.length()
    }

    #[inline(always)]
    pub fn length(&self) -> f64 {
        return self.length_squared().sqrt();
    }

    #[inline(always)]
    pub fn min(&self, other: &Vec3) -> Vec3 {
        return Vec3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        };
    }

    #[inline(always)]
    pub fn max(&self, other: &Vec3) -> Vec3 {
        return Vec3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        };
    }

    #[inline(always)]
    pub fn abs(&self) -> Vec3 {
        return Vec3 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        };
    }

    #[inline(always)]
    pub fn length_squared(&self) -> f64 {
        return self.x * self.x + self.y * self.y + self.z * self.z;
    }

    #[inline(always)]
    pub fn dot(&self, b: &Vec3) -> f64 {
        return self.x * b.x + self.y * b.y + self.z * b.z;
    }

    #[inline(always)]
    pub fn cross(&self, b: &Vec3) -> Vec3 {
        return Vec3 {
            x: self.y * b.z - self.z * b.y,
            y: self.z * b.x - self.x * b.z,
            z: self.x * b.y - self.y * b.x,
        };
    }

    #[inline(always)]
    pub fn default_random<T: rand::Rng>(r: &mut T) -> Vec3 {
        return Vec3 {
            x: r.gen_range(0.0..1.0),
            y: r.gen_range(0.0..1.0),
            z: r.gen_range(0.0..1.0),
        };
    }

    #[inline(always)]
    pub fn random<T: rand::Rng>(r: &mut T, range: Range<f64>) -> Vec3 {
        return Vec3 {
            x: r.gen_range(range.clone()),
            y: r.gen_range(range.clone()),
            z: r.gen_range(range),
        };
    }

    #[inline(always)]
    pub fn near_zero(&self) -> bool {
        const S: f64 = 1e-8;
        return (self.x.abs() < S) && (self.y.abs() < S) && (self.z.abs() < S);
    }

    pub fn axis(&self, axis: usize) -> f64 {
        return match axis {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!(),
        };
    }

    #[inline(always)]
    pub fn reflect(&self, n: &Vec3) -> Vec3 {
        return self - 2.0 * Vec3::dot(self, n) * n;
    }

    #[inline(always)]
    pub fn refract(&self, n: &Vec3, etai_over_etat: f64) -> Vec3 {
        let cos_theta = ((-1.0) * self).dot(n).min(1.0);
        let r_out_perp = etai_over_etat * (self + cos_theta * n);
        let r_out_parallel = -(1.0 - r_out_perp.length().powi(2)).abs().sqrt() * n;
        r_out_perp + r_out_parallel
    }
}

impl From<f64> for Vec3 {
    #[inline(always)]
    fn from(value: f64) -> Self {
        Vec3 {
            x: value,
            y: value,
            z: value,
        }
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => &0.0,
        }
    }
}

impl IndexMut<usize> for Vec3 {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("index not in range for vec3"),
        }
    }
}

unsafe impl Send for Vec3 {}
unsafe impl Sync for Vec3 {}

impl std::fmt::Display for Vec3 {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.x)?;
        write!(f, "{:?}", self.x)?;
        write!(f, "{:?}", self.x)?;
        return Ok(());
    }
}

// This macro helps us implement math operators on Vector3
// in such a way that it handles binary operators on any
// combination of Vec3, &Vec3 and f64.
macro_rules! impl_binary_operations {
    // $VectorType is something like `Vec3`
    // $Operation is something like `Add`
    // $op_fn is something like `add`
    // $op_symbol is something like `+`
    ($VectorType:ident $Operation:ident $op_fn:ident $op_symbol:tt) => {
        // Implement a + b where a and b are both of type &VectorType.
        // Lower down we'll implement cases where either a or b - or both
        // - are values by forwarding through to this implementation.
        impl<'a, 'b> $Operation<&'a $VectorType> for &'b $VectorType {
            type Output = $VectorType;
            #[inline(always)]
            fn $op_fn(self, other: &'a $VectorType) -> $VectorType {
                $VectorType {
                    x: self.x $op_symbol other.x,
                    y: self.y $op_symbol other.y,
                    z: self.z $op_symbol other.z,
                }
            }
        }

        // Implement a + b for the cases...
        //
        //   a: $VectorType,  b: &$VectorType
        //   a: &$VectorType, b: $VectorType
        //   a: $VectorType, b: $VectorType
        //
        // In each case we forward through to the implementation above.
        impl $Operation<$VectorType> for $VectorType {
            type Output = $VectorType;

            #[inline(always)]
            fn $op_fn(self, other: $VectorType) -> $VectorType {
                &self $op_symbol &other
            }
        }

        impl<'a> $Operation<&'a $VectorType> for $VectorType {
            type Output = $VectorType;

            #[inline(always)]
            fn $op_fn(self, other: &'a $VectorType) -> $VectorType {
                &self $op_symbol other
            }
        }

        impl<'a> $Operation<$VectorType> for &'a $VectorType {
            type Output = $VectorType;

            #[inline(always)]
            fn $op_fn(self, other: $VectorType) -> $VectorType {
                self $op_symbol &other
            }
        }

        // Implement a + b where a is type &$VectorType and b is type f64
        impl<'a> $Operation<f64> for &'a $VectorType {
            type Output = $VectorType;

			#[inline(always)]
            fn $op_fn(self, other: f64) -> $VectorType {
                $VectorType {
                    x: self.x $op_symbol other,
                    y: self.y $op_symbol other,
                    z: self.z $op_symbol other
                }
            }
        }

        // Implement a + b where...
        //
        // a is $VectorType and b is f64
        // a is f64 and b is $VectorType
        // a is f64 and b is &$VectorType
        //
        // In each case we forward the logic to the implementation
        // above.
        impl $Operation<f64> for $VectorType {
            type Output = $VectorType;

            #[inline(always)]
            fn $op_fn(self, other: f64) -> $VectorType {
                &self $op_symbol other
            }
        }

        impl $Operation<$VectorType> for f64 {
            type Output = $VectorType;

            #[inline(always)]
            fn $op_fn(self, other: $VectorType) -> $VectorType {
                &other $op_symbol self
            }
        }

        impl<'a> $Operation<&'a $VectorType> for f64 {
            type Output = $VectorType;

            #[inline(always)]
            fn $op_fn(self, other: &'a $VectorType) -> $VectorType {
                other $op_symbol self
            }
        }
    };
}

// It also implements unary operators like - a where a is of
// type Vec3 or &Vec3.
macro_rules! impl_unary_operations {
    // $VectorType is something like `Vec3`
    // $Operation is something like `Neg`
    // $op_fn is something like `neg`
    // $op_symbol is something like `-`
    ($VectorType:ident $Operation:ident $op_fn:ident $op_symbol:tt) => {

        // Implement the unary operator for references
        impl<'a> $Operation for &'a $VectorType {
            type Output = $VectorType;

            #[inline(always)]
            fn $op_fn(self) -> Vec3 {
                $VectorType {
                    x: $op_symbol self.x,
                    y: $op_symbol self.y,
                    z: $op_symbol self.z,
                }
            }
        }

        // Have the operator on values forward through to the implementation
        // above
        impl $Operation for $VectorType {
            type Output = $VectorType;

            #[inline(always)]
            fn $op_fn(self) -> Vec3 {
                $op_symbol &self
            }
        }
    };
}

// Implement add-assignment operators like a += b where a and
// b is either &Vec3 or Vec3 (in this case a is always of type
// &mut Vec3).
macro_rules! impl_op_assign {
    // $VectorType is something like `Vec3`
    // $OperationAssign is something like `AddAssign`
    // $op_fn is something like `add_assign`
    // $op_symbol is something like `+=`
    ($VectorType:ident $OperationAssign:ident $op_fn:ident $op_symbol:tt) => {
        // Implement $OperationAssign for RHS &Vec3
        impl<'a> $OperationAssign<&'a $VectorType> for $VectorType {
            #[inline(always)]
            fn $op_fn(&mut self, other: &'a $VectorType) {
                *self = $VectorType {
                    x: self.x $op_symbol other.x,
                    y: self.y $op_symbol other.y,
                    z: self.z $op_symbol other.z,
                };
            }
        }

        // Implement $OperationAssign for RHS Vec3 by forwarding through to the
        // implementation above
        impl $OperationAssign for $VectorType {
            #[inline(always)]
            fn $op_fn(&mut self, other: $VectorType) {
                *self = *self $op_symbol &other
            }
        }
    };
}

impl_binary_operations!(Vec3 Add add +);
impl_op_assign!(Vec3 AddAssign add_assign +);

impl_binary_operations!(Vec3 Sub sub -);
impl_op_assign!(Vec3 SubAssign sub_assign -);
impl_unary_operations!(Vec3 Neg neg -);

impl_binary_operations!(Vec3 Mul mul *);
impl_op_assign!(Vec3 MulAssign mul_assign *);

impl_binary_operations!(Vec3 Div div /);
impl_op_assign!(Vec3 DivAssign div_assign /);

#[cfg(test)]
mod tests {
    use super::Vec3;

    #[test]
    fn construct() {
        let data = Vec3::new(1.6, 100066666.6732, -262.0);
        assert_eq!(data.x, 1.6);
        assert_eq!(data.y, 100066666.6732);
        assert_eq!(data.z, -262.0);
    }

    #[test]
    fn add() {}

    #[test]
    fn multiply() {}

    #[test]
    fn lenght() {}

    #[test]
    fn lenght_squared() {}

    #[test]
    fn normalizing() {}
}
