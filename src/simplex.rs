use bevy::prelude::*;

use crate::{
    NoiseSeed,
    config::SimplexConfig,
    hash::{hash2, hash3, hash4},
    sample::{NoiseRange, NoiseSource, RangeSemantics},
};

const GRAD3: [[f32; 3]; 12] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
];

const GRAD4: [[f32; 4]; 32] = [
    [0.0, 1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0, -1.0],
    [0.0, 1.0, -1.0, 1.0],
    [0.0, 1.0, -1.0, -1.0],
    [0.0, -1.0, 1.0, 1.0],
    [0.0, -1.0, 1.0, -1.0],
    [0.0, -1.0, -1.0, 1.0],
    [0.0, -1.0, -1.0, -1.0],
    [1.0, 0.0, 1.0, 1.0],
    [1.0, 0.0, 1.0, -1.0],
    [1.0, 0.0, -1.0, 1.0],
    [1.0, 0.0, -1.0, -1.0],
    [-1.0, 0.0, 1.0, 1.0],
    [-1.0, 0.0, 1.0, -1.0],
    [-1.0, 0.0, -1.0, 1.0],
    [-1.0, 0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0, 1.0],
    [1.0, 1.0, 0.0, -1.0],
    [1.0, -1.0, 0.0, 1.0],
    [1.0, -1.0, 0.0, -1.0],
    [-1.0, 1.0, 0.0, 1.0],
    [-1.0, 1.0, 0.0, -1.0],
    [-1.0, -1.0, 0.0, 1.0],
    [-1.0, -1.0, 0.0, -1.0],
    [1.0, 1.0, 1.0, 0.0],
    [1.0, 1.0, -1.0, 0.0],
    [1.0, -1.0, 1.0, 0.0],
    [1.0, -1.0, -1.0, 0.0],
    [-1.0, 1.0, 1.0, 0.0],
    [-1.0, 1.0, -1.0, 0.0],
    [-1.0, -1.0, 1.0, 0.0],
    [-1.0, -1.0, -1.0, 0.0],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct Simplex {
    pub seed: NoiseSeed,
}

impl Simplex {
    #[must_use]
    pub const fn new(seed: NoiseSeed) -> Self {
        Self { seed }
    }
}

impl From<SimplexConfig> for Simplex {
    fn from(config: SimplexConfig) -> Self {
        Self::new(config.seed)
    }
}

impl NoiseSource<Vec2> for Simplex {
    fn sample(&self, point: Vec2) -> f32 {
        const F2: f32 = 0.366_025_42;
        const G2: f32 = 0.211_324_87;

        let s = (point.x + point.y) * F2;
        let i = (point.x + s).floor() as i32;
        let j = (point.y + s).floor() as i32;
        let t = (i + j) as f32 * G2;
        let x0 = point.x - (i as f32 - t);
        let y0 = point.y - (j as f32 - t);

        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
        let x1 = x0 - i1 as f32 + G2;
        let y1 = y0 - j1 as f32 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        let n0 = simplex_contribution2(hash2(self.seed, i, j), x0, y0);
        let n1 = simplex_contribution2(hash2(self.seed, i + i1, j + j1), x1, y1);
        let n2 = simplex_contribution2(hash2(self.seed, i + 1, j + 1), x2, y2);
        70.0 * (n0 + n1 + n2)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate)
    }
}

impl NoiseSource<Vec3> for Simplex {
    fn sample(&self, point: Vec3) -> f32 {
        const F3: f32 = 1.0 / 3.0;
        const G3: f32 = 1.0 / 6.0;

        let s = (point.x + point.y + point.z) * F3;
        let i = (point.x + s).floor() as i32;
        let j = (point.y + s).floor() as i32;
        let k = (point.z + s).floor() as i32;
        let t = (i + j + k) as f32 * G3;
        let x0 = point.x - (i as f32 - t);
        let y0 = point.y - (j as f32 - t);
        let z0 = point.z - (k as f32 - t);

        let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
            if y0 >= z0 {
                (1, 0, 0, 1, 1, 0)
            } else if x0 >= z0 {
                (1, 0, 0, 1, 0, 1)
            } else {
                (0, 0, 1, 1, 0, 1)
            }
        } else if y0 < z0 {
            (0, 0, 1, 0, 1, 1)
        } else if x0 < z0 {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        };

        let x1 = x0 - i1 as f32 + G3;
        let y1 = y0 - j1 as f32 + G3;
        let z1 = z0 - k1 as f32 + G3;
        let x2 = x0 - i2 as f32 + 2.0 * G3;
        let y2 = y0 - j2 as f32 + 2.0 * G3;
        let z2 = z0 - k2 as f32 + 2.0 * G3;
        let x3 = x0 - 1.0 + 3.0 * G3;
        let y3 = y0 - 1.0 + 3.0 * G3;
        let z3 = z0 - 1.0 + 3.0 * G3;

        let n0 = simplex_contribution3(hash3(self.seed, i, j, k), x0, y0, z0);
        let n1 = simplex_contribution3(hash3(self.seed, i + i1, j + j1, k + k1), x1, y1, z1);
        let n2 = simplex_contribution3(hash3(self.seed, i + i2, j + j2, k + k2), x2, y2, z2);
        let n3 = simplex_contribution3(hash3(self.seed, i + 1, j + 1, k + 1), x3, y3, z3);
        32.0 * (n0 + n1 + n2 + n3)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate)
    }
}

impl NoiseSource<Vec4> for Simplex {
    fn sample(&self, point: Vec4) -> f32 {
        const F4: f32 = 0.309_016_97;
        const G4: f32 = 0.138_196_6;

        let s = (point.x + point.y + point.z + point.w) * F4;
        let i = (point.x + s).floor() as i32;
        let j = (point.y + s).floor() as i32;
        let k = (point.z + s).floor() as i32;
        let l = (point.w + s).floor() as i32;
        let t = (i + j + k + l) as f32 * G4;
        let x0 = point.x - (i as f32 - t);
        let y0 = point.y - (j as f32 - t);
        let z0 = point.z - (k as f32 - t);
        let w0 = point.w - (l as f32 - t);

        let mut rankx = 0;
        let mut ranky = 0;
        let mut rankz = 0;
        let mut rankw = 0;

        if x0 > y0 {
            rankx += 1;
        } else {
            ranky += 1;
        }
        if x0 > z0 {
            rankx += 1;
        } else {
            rankz += 1;
        }
        if x0 > w0 {
            rankx += 1;
        } else {
            rankw += 1;
        }
        if y0 > z0 {
            ranky += 1;
        } else {
            rankz += 1;
        }
        if y0 > w0 {
            ranky += 1;
        } else {
            rankw += 1;
        }
        if z0 > w0 {
            rankz += 1;
        } else {
            rankw += 1;
        }

        let i1 = i32::from(rankx >= 3);
        let j1 = i32::from(ranky >= 3);
        let k1 = i32::from(rankz >= 3);
        let l1 = i32::from(rankw >= 3);

        let i2 = i32::from(rankx >= 2);
        let j2 = i32::from(ranky >= 2);
        let k2 = i32::from(rankz >= 2);
        let l2 = i32::from(rankw >= 2);

        let i3 = i32::from(rankx >= 1);
        let j3 = i32::from(ranky >= 1);
        let k3 = i32::from(rankz >= 1);
        let l3 = i32::from(rankw >= 1);

        let x1 = x0 - i1 as f32 + G4;
        let y1 = y0 - j1 as f32 + G4;
        let z1 = z0 - k1 as f32 + G4;
        let w1 = w0 - l1 as f32 + G4;

        let x2 = x0 - i2 as f32 + 2.0 * G4;
        let y2 = y0 - j2 as f32 + 2.0 * G4;
        let z2 = z0 - k2 as f32 + 2.0 * G4;
        let w2 = w0 - l2 as f32 + 2.0 * G4;

        let x3 = x0 - i3 as f32 + 3.0 * G4;
        let y3 = y0 - j3 as f32 + 3.0 * G4;
        let z3 = z0 - k3 as f32 + 3.0 * G4;
        let w3 = w0 - l3 as f32 + 3.0 * G4;

        let x4 = x0 - 1.0 + 4.0 * G4;
        let y4 = y0 - 1.0 + 4.0 * G4;
        let z4 = z0 - 1.0 + 4.0 * G4;
        let w4 = w0 - 1.0 + 4.0 * G4;

        let n0 = simplex_contribution4(hash4(self.seed, i, j, k, l), x0, y0, z0, w0);
        let n1 = simplex_contribution4(
            hash4(self.seed, i + i1, j + j1, k + k1, l + l1),
            x1,
            y1,
            z1,
            w1,
        );
        let n2 = simplex_contribution4(
            hash4(self.seed, i + i2, j + j2, k + k2, l + l2),
            x2,
            y2,
            z2,
            w2,
        );
        let n3 = simplex_contribution4(
            hash4(self.seed, i + i3, j + j3, k + k3, l + l3),
            x3,
            y3,
            z3,
            w3,
        );
        let n4 =
            simplex_contribution4(hash4(self.seed, i + 1, j + 1, k + 1, l + 1), x4, y4, z4, w4);
        27.0 * (n0 + n1 + n2 + n3 + n4)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate)
    }
}

#[inline]
fn simplex_contribution2(hash: u32, x: f32, y: f32) -> f32 {
    let mut t = 0.5 - x * x - y * y;
    if t < 0.0 {
        return 0.0;
    }
    let grad = GRAD3[(hash as usize) % GRAD3.len()];
    t *= t;
    t * t * (grad[0] * x + grad[1] * y)
}

#[inline]
fn simplex_contribution3(hash: u32, x: f32, y: f32, z: f32) -> f32 {
    let mut t = 0.6 - x * x - y * y - z * z;
    if t < 0.0 {
        return 0.0;
    }
    let grad = GRAD3[(hash as usize) % GRAD3.len()];
    t *= t;
    t * t * (grad[0] * x + grad[1] * y + grad[2] * z)
}

#[inline]
fn simplex_contribution4(hash: u32, x: f32, y: f32, z: f32, w: f32) -> f32 {
    let mut t = 0.6 - x * x - y * y - z * z - w * w;
    if t < 0.0 {
        return 0.0;
    }
    let grad = GRAD4[(hash as usize) % GRAD4.len()];
    t *= t;
    t * t * (grad[0] * x + grad[1] * y + grad[2] * z + grad[3] * w)
}
