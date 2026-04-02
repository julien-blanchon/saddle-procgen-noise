use bevy::prelude::*;

use crate::{
    NoiseSeed,
    config::PerlinConfig,
    hash::{hash2, hash3, hash4},
    sample::{NoiseRange, NoiseSource, RangeSemantics},
};

const GRADIENTS_2D: [[f32; 2]; 8] = [
    [1.0, 0.0],
    [-1.0, 0.0],
    [0.0, 1.0],
    [0.0, -1.0],
    [0.707_106_77, 0.707_106_77],
    [-0.707_106_77, 0.707_106_77],
    [0.707_106_77, -0.707_106_77],
    [-0.707_106_77, -0.707_106_77],
];

const GRADIENTS_3D: [[f32; 3]; 12] = [
    [0.707_106_77, 0.707_106_77, 0.0],
    [-0.707_106_77, 0.707_106_77, 0.0],
    [0.707_106_77, -0.707_106_77, 0.0],
    [-0.707_106_77, -0.707_106_77, 0.0],
    [0.707_106_77, 0.0, 0.707_106_77],
    [-0.707_106_77, 0.0, 0.707_106_77],
    [0.707_106_77, 0.0, -0.707_106_77],
    [-0.707_106_77, 0.0, -0.707_106_77],
    [0.0, 0.707_106_77, 0.707_106_77],
    [0.0, -0.707_106_77, 0.707_106_77],
    [0.0, 0.707_106_77, -0.707_106_77],
    [0.0, -0.707_106_77, -0.707_106_77],
];

const GRADIENTS_4D: [[f32; 4]; 32] = [
    [0.0, 0.577_350_26, 0.577_350_26, 0.577_350_26],
    [0.0, 0.577_350_26, 0.577_350_26, -0.577_350_26],
    [0.0, 0.577_350_26, -0.577_350_26, 0.577_350_26],
    [0.0, 0.577_350_26, -0.577_350_26, -0.577_350_26],
    [0.0, -0.577_350_26, 0.577_350_26, 0.577_350_26],
    [0.0, -0.577_350_26, 0.577_350_26, -0.577_350_26],
    [0.0, -0.577_350_26, -0.577_350_26, 0.577_350_26],
    [0.0, -0.577_350_26, -0.577_350_26, -0.577_350_26],
    [0.577_350_26, 0.0, 0.577_350_26, 0.577_350_26],
    [0.577_350_26, 0.0, 0.577_350_26, -0.577_350_26],
    [0.577_350_26, 0.0, -0.577_350_26, 0.577_350_26],
    [0.577_350_26, 0.0, -0.577_350_26, -0.577_350_26],
    [-0.577_350_26, 0.0, 0.577_350_26, 0.577_350_26],
    [-0.577_350_26, 0.0, 0.577_350_26, -0.577_350_26],
    [-0.577_350_26, 0.0, -0.577_350_26, 0.577_350_26],
    [-0.577_350_26, 0.0, -0.577_350_26, -0.577_350_26],
    [0.577_350_26, 0.577_350_26, 0.0, 0.577_350_26],
    [0.577_350_26, 0.577_350_26, 0.0, -0.577_350_26],
    [0.577_350_26, -0.577_350_26, 0.0, 0.577_350_26],
    [0.577_350_26, -0.577_350_26, 0.0, -0.577_350_26],
    [-0.577_350_26, 0.577_350_26, 0.0, 0.577_350_26],
    [-0.577_350_26, 0.577_350_26, 0.0, -0.577_350_26],
    [-0.577_350_26, -0.577_350_26, 0.0, 0.577_350_26],
    [-0.577_350_26, -0.577_350_26, 0.0, -0.577_350_26],
    [0.577_350_26, 0.577_350_26, 0.577_350_26, 0.0],
    [0.577_350_26, 0.577_350_26, -0.577_350_26, 0.0],
    [0.577_350_26, -0.577_350_26, 0.577_350_26, 0.0],
    [0.577_350_26, -0.577_350_26, -0.577_350_26, 0.0],
    [-0.577_350_26, 0.577_350_26, 0.577_350_26, 0.0],
    [-0.577_350_26, 0.577_350_26, -0.577_350_26, 0.0],
    [-0.577_350_26, -0.577_350_26, 0.577_350_26, 0.0],
    [-0.577_350_26, -0.577_350_26, -0.577_350_26, 0.0],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct Perlin {
    pub seed: NoiseSeed,
}

impl Perlin {
    #[must_use]
    pub const fn new(seed: NoiseSeed) -> Self {
        Self { seed }
    }
}

impl From<PerlinConfig> for Perlin {
    fn from(config: PerlinConfig) -> Self {
        Self::new(config.seed)
    }
}

#[must_use]
pub fn fade_curve(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl NoiseSource<Vec2> for Perlin {
    fn sample(&self, point: Vec2) -> f32 {
        let cell = point.floor().as_ivec2();
        let local = point - cell.as_vec2();
        let u = fade_curve(local.x);
        let v = fade_curve(local.y);

        let gradients = [
            gradient_2d(hash2(self.seed, cell.x, cell.y)),
            gradient_2d(hash2(self.seed, cell.x + 1, cell.y)),
            gradient_2d(hash2(self.seed, cell.x, cell.y + 1)),
            gradient_2d(hash2(self.seed, cell.x + 1, cell.y + 1)),
        ];

        let dots = [
            gradients[0].dot(local),
            gradients[1].dot(local - Vec2::X),
            gradients[2].dot(local - Vec2::Y),
            gradients[3].dot(local - Vec2::ONE),
        ];

        let nx0 = lerp(dots[0], dots[1], u);
        let nx1 = lerp(dots[2], dots[3], u);
        lerp(nx0, nx1, v)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate)
    }
}

impl NoiseSource<Vec3> for Perlin {
    fn sample(&self, point: Vec3) -> f32 {
        let cell = point.floor().as_ivec3();
        let local = point - cell.as_vec3();
        let u = fade_curve(local.x);
        let v = fade_curve(local.y);
        let w = fade_curve(local.z);

        let mut dots = [0.0; 8];
        for (corner, dot) in dots.iter_mut().enumerate() {
            let offset = IVec3::new(
                (corner & 1) as i32,
                ((corner >> 1) & 1) as i32,
                ((corner >> 2) & 1) as i32,
            );
            let gradient = gradient_3d(hash3(
                self.seed,
                cell.x + offset.x,
                cell.y + offset.y,
                cell.z + offset.z,
            ));
            let delta = local - offset.as_vec3();
            *dot = gradient.dot(delta);
        }

        let x00 = lerp(dots[0], dots[1], u);
        let x10 = lerp(dots[2], dots[3], u);
        let x01 = lerp(dots[4], dots[5], u);
        let x11 = lerp(dots[6], dots[7], u);
        let y0 = lerp(x00, x10, v);
        let y1 = lerp(x01, x11, v);
        lerp(y0, y1, w)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate)
    }
}

impl NoiseSource<Vec4> for Perlin {
    fn sample(&self, point: Vec4) -> f32 {
        let cell = point.floor().as_ivec4();
        let local = point - cell.as_vec4();
        let u = fade_curve(local.x);
        let v = fade_curve(local.y);
        let w = fade_curve(local.z);
        let q = fade_curve(local.w);

        let mut dots = [0.0; 16];
        for (corner, dot) in dots.iter_mut().enumerate() {
            let offset = IVec4::new(
                (corner & 1) as i32,
                ((corner >> 1) & 1) as i32,
                ((corner >> 2) & 1) as i32,
                ((corner >> 3) & 1) as i32,
            );
            let gradient = gradient_4d(hash4(
                self.seed,
                cell.x + offset.x,
                cell.y + offset.y,
                cell.z + offset.z,
                cell.w + offset.w,
            ));
            let delta = local - offset.as_vec4();
            *dot = gradient.dot(delta);
        }

        let interp = |a: f32, b: f32, t: f32| lerp(a, b, t);

        let x000 = interp(dots[0], dots[1], u);
        let x100 = interp(dots[2], dots[3], u);
        let x010 = interp(dots[4], dots[5], u);
        let x110 = interp(dots[6], dots[7], u);
        let x001 = interp(dots[8], dots[9], u);
        let x101 = interp(dots[10], dots[11], u);
        let x011 = interp(dots[12], dots[13], u);
        let x111 = interp(dots[14], dots[15], u);

        let y00 = interp(x000, x100, v);
        let y10 = interp(x010, x110, v);
        let y01 = interp(x001, x101, v);
        let y11 = interp(x011, x111, v);

        let z0 = interp(y00, y10, w);
        let z1 = interp(y01, y11, w);
        interp(z0, z1, q)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate)
    }
}

#[inline]
fn gradient_2d(hash: u32) -> Vec2 {
    let gradient = GRADIENTS_2D[(hash as usize) % GRADIENTS_2D.len()];
    Vec2::new(gradient[0], gradient[1])
}

#[inline]
fn gradient_3d(hash: u32) -> Vec3 {
    let gradient = GRADIENTS_3D[(hash as usize) % GRADIENTS_3D.len()];
    Vec3::new(gradient[0], gradient[1], gradient[2])
}

#[inline]
fn gradient_4d(hash: u32) -> Vec4 {
    let gradient = GRADIENTS_4D[(hash as usize) % GRADIENTS_4D.len()];
    Vec4::new(gradient[0], gradient[1], gradient[2], gradient[3])
}
