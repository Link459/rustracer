use rand::{ RngExt};

use crate::{vec3::Vec3, Float};

const MAX_PERLIN: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct Perlin {
    ran_vec: [Vec3; MAX_PERLIN],
    perm_x: [usize; MAX_PERLIN],
    perm_y: [usize; MAX_PERLIN],
    perm_z: [usize; MAX_PERLIN],
}

impl Default for Perlin {
    fn default() -> Self {
        return Self::new();
    }
}

impl Perlin {
    pub fn new() -> Self {
        let mut ran_vec = [Vec3::ZERO; MAX_PERLIN];
        let mut rng = rand::rng();
        for i in ran_vec.iter_mut().take(MAX_PERLIN) {
            *i = Vec3::random(&mut rng, -1.0..1.0);
        }

        return Self {
            perm_x: generate_perm(),
            perm_y: generate_perm(),
            perm_z: generate_perm(),
            ran_vec,
        };
    }

    pub fn noise(&self, p: &Vec3) -> Float {
        let mut u = p.x - p.x.floor();
        let mut v = p.y - p.y.floor();
        let mut w = p.z - p.z.floor();
        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);

        let i = p.x.floor() as usize;
        let j = p.y.floor() as usize;
        let k = p.z.floor() as usize;

        let mut c = [[[Vec3::ZERO; 2]; 2]; 2];

        #[allow(clippy::needless_range_loop)]
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ran_vec[self.perm_x[(i + di) & 255]
                        ^ self.perm_y[(j + dj) & 255]
                        ^ self.perm_z[(k + dk) & 255]];
                }
            }
        }
        return trilinear_interp(&c, u, v, w);
    }

    pub fn turb(&self, p: &Vec3, depth: u32) -> Float {
        let mut accum = 0.0;
        let mut t_p = *p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&t_p);
            weight *= 0.5;
            t_p = t_p * 2.0;
        }
        return accum.abs();
    }
}
fn generate_perm() -> [usize; MAX_PERLIN] {
    let mut p = [0; MAX_PERLIN];
    for (i, e) in p.iter_mut().enumerate() {
        *e = i;
    }
    permute(&mut p, 256);
    return p;
}

fn permute(p: &mut [usize; MAX_PERLIN], n: usize) {
    let mut rng = rand::rng();
    for i in (0..n).rev() {
        let target = rng.random_range(0..(i + 1));
        p.swap(i, target);
    }
}

fn trilinear_interp(c: &[[[Vec3; 2]; 2]; 2], u: Float, v: Float, w: Float) -> Float {
    let uu = u * u * (3.0 - 2.0 * u);
    let vv = v * v * (3.0 - 2.0 * v);
    let ww = w * w * (3.0 - 2.0 * w);
    let mut accum = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let weight = Vec3::new(u - i as Float, v - j as Float, w - k as Float);
                accum += (i as Float * uu + (1 - i) as Float * (1.0 - uu))
                    * (j as Float * vv + (1 - j) as Float * (1.0 - vv))
                    * (k as Float * ww + (1 - k) as Float * (1.0 - ww))
                    * c[i][j][k].dot(&weight);
            }
        }
    }
    return accum;
}
