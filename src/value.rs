use bevy::prelude::*;

use crate::{
    NoiseSeed,
    config::ValueConfig,
    hash::{hash2, hash3, hash4, unit_float},
    perlin::fade_curve,
    sample::{NoiseRange, NoiseSource, RangeSemantics},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct Value {
    pub seed: NoiseSeed,
}

impl Value {
    #[must_use]
    pub const fn new(seed: NoiseSeed) -> Self {
        Self { seed }
    }
}

impl From<ValueConfig> for Value {
    fn from(config: ValueConfig) -> Self {
        Self::new(config.seed)
    }
}

impl NoiseSource<Vec2> for Value {
    fn sample(&self, point: Vec2) -> f32 {
        let cell = point.floor().as_ivec2();
        let local = point - cell.as_vec2();
        let u = fade_curve(local.x);
        let v = fade_curve(local.y);

        let values = [
            lattice_value(hash2(self.seed, cell.x, cell.y)),
            lattice_value(hash2(self.seed, cell.x + 1, cell.y)),
            lattice_value(hash2(self.seed, cell.x, cell.y + 1)),
            lattice_value(hash2(self.seed, cell.x + 1, cell.y + 1)),
        ];

        let nx0 = values[0].lerp(values[1], u);
        let nx1 = values[2].lerp(values[3], u);
        nx0.lerp(nx1, v)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Strict)
    }
}

impl NoiseSource<Vec3> for Value {
    fn sample(&self, point: Vec3) -> f32 {
        let cell = point.floor().as_ivec3();
        let local = point - cell.as_vec3();
        let u = fade_curve(local.x);
        let v = fade_curve(local.y);
        let w = fade_curve(local.z);

        let mut values = [0.0; 8];
        for (corner, value) in values.iter_mut().enumerate() {
            let offset = IVec3::new(
                (corner & 1) as i32,
                ((corner >> 1) & 1) as i32,
                ((corner >> 2) & 1) as i32,
            );
            *value = lattice_value(hash3(
                self.seed,
                cell.x + offset.x,
                cell.y + offset.y,
                cell.z + offset.z,
            ));
        }

        let x00 = values[0].lerp(values[1], u);
        let x10 = values[2].lerp(values[3], u);
        let x01 = values[4].lerp(values[5], u);
        let x11 = values[6].lerp(values[7], u);
        let y0 = x00.lerp(x10, v);
        let y1 = x01.lerp(x11, v);
        y0.lerp(y1, w)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Strict)
    }
}

impl NoiseSource<Vec4> for Value {
    fn sample(&self, point: Vec4) -> f32 {
        let cell = point.floor().as_ivec4();
        let local = point - cell.as_vec4();
        let u = fade_curve(local.x);
        let v = fade_curve(local.y);
        let w = fade_curve(local.z);
        let q = fade_curve(local.w);

        let mut values = [0.0; 16];
        for (corner, value) in values.iter_mut().enumerate() {
            let offset = IVec4::new(
                (corner & 1) as i32,
                ((corner >> 1) & 1) as i32,
                ((corner >> 2) & 1) as i32,
                ((corner >> 3) & 1) as i32,
            );
            *value = lattice_value(hash4(
                self.seed,
                cell.x + offset.x,
                cell.y + offset.y,
                cell.z + offset.z,
                cell.w + offset.w,
            ));
        }

        let x000 = values[0].lerp(values[1], u);
        let x100 = values[2].lerp(values[3], u);
        let x010 = values[4].lerp(values[5], u);
        let x110 = values[6].lerp(values[7], u);
        let x001 = values[8].lerp(values[9], u);
        let x101 = values[10].lerp(values[11], u);
        let x011 = values[12].lerp(values[13], u);
        let x111 = values[14].lerp(values[15], u);

        let y00 = x000.lerp(x100, v);
        let y10 = x010.lerp(x110, v);
        let y01 = x001.lerp(x101, v);
        let y11 = x011.lerp(x111, v);

        let z0 = y00.lerp(y10, w);
        let z1 = y01.lerp(y11, w);
        z0.lerp(z1, q)
    }

    fn native_range(&self) -> NoiseRange {
        NoiseRange::new(-1.0, 1.0, RangeSemantics::Strict)
    }
}

fn lattice_value(hash: u32) -> f32 {
    unit_float(hash) * 2.0 - 1.0
}

#[cfg(test)]
#[path = "value_tests.rs"]
mod tests;
