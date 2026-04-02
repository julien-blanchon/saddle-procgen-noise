use bevy::prelude::*;

use crate::{
    NoiseSeed,
    config::{WorleyConfig, WorleyDistanceMetric, WorleyReturnType},
    hash::{hash2, hash3, unit_float},
    sample::{NoiseRange, NoiseSource, RangeSemantics},
};

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct Worley {
    pub config: WorleyConfig,
}

impl Worley {
    #[must_use]
    pub const fn new(config: WorleyConfig) -> Self {
        Self { config }
    }
}

impl From<WorleyConfig> for Worley {
    fn from(config: WorleyConfig) -> Self {
        Self::new(config)
    }
}

impl NoiseSource<Vec2> for Worley {
    fn sample(&self, point: Vec2) -> f32 {
        sample_worley_2d(self.config, point)
    }

    fn native_range(&self) -> NoiseRange {
        worley_range(self.config.distance, self.config.return_type, 2)
    }
}

impl NoiseSource<Vec3> for Worley {
    fn sample(&self, point: Vec3) -> f32 {
        sample_worley_3d(self.config, point)
    }

    fn native_range(&self) -> NoiseRange {
        worley_range(self.config.distance, self.config.return_type, 3)
    }
}

#[must_use]
fn sample_worley_2d(config: WorleyConfig, point: Vec2) -> f32 {
    let base = point.floor().as_ivec2();
    let mut f1 = f32::INFINITY;
    let mut f2 = f32::INFINITY;

    for oy in -1..=1 {
        for ox in -1..=1 {
            let cell = base + IVec2::new(ox, oy);
            let feature = feature_point_2d(config.seed, cell, config.jitter);
            let distance = distance_2d(config.distance, point - feature);
            if distance < f1 {
                f2 = f1;
                f1 = distance;
            } else if distance < f2 {
                f2 = distance;
            }
        }
    }

    match config.return_type {
        WorleyReturnType::F1 => f1,
        WorleyReturnType::F2 => f2,
        WorleyReturnType::F2MinusF1 => (f2 - f1).max(0.0),
    }
}

#[must_use]
fn sample_worley_3d(config: WorleyConfig, point: Vec3) -> f32 {
    let base = point.floor().as_ivec3();
    let mut f1 = f32::INFINITY;
    let mut f2 = f32::INFINITY;

    for oz in -1..=1 {
        for oy in -1..=1 {
            for ox in -1..=1 {
                let cell = base + IVec3::new(ox, oy, oz);
                let feature = feature_point_3d(config.seed, cell, config.jitter);
                let distance = distance_3d(config.distance, point - feature);
                if distance < f1 {
                    f2 = f1;
                    f1 = distance;
                } else if distance < f2 {
                    f2 = distance;
                }
            }
        }
    }

    match config.return_type {
        WorleyReturnType::F1 => f1,
        WorleyReturnType::F2 => f2,
        WorleyReturnType::F2MinusF1 => (f2 - f1).max(0.0),
    }
}

#[must_use]
fn feature_point_2d(seed: NoiseSeed, cell: IVec2, jitter: f32) -> Vec2 {
    let jitter = jitter.clamp(0.0, 1.0);
    let center = cell.as_vec2() + Vec2::splat(0.5);
    let jitter_x = unit_float(hash2(seed, cell.x, cell.y)) - 0.5;
    let jitter_y = unit_float(hash2(seed.split(0xA341_316C), cell.x, cell.y)) - 0.5;
    center + Vec2::new(jitter_x, jitter_y) * jitter
}

#[must_use]
fn feature_point_3d(seed: NoiseSeed, cell: IVec3, jitter: f32) -> Vec3 {
    let jitter = jitter.clamp(0.0, 1.0);
    let center = cell.as_vec3() + Vec3::splat(0.5);
    let jitter_x = unit_float(hash3(seed, cell.x, cell.y, cell.z)) - 0.5;
    let jitter_y = unit_float(hash3(seed.split(0xC801_3EA4), cell.x, cell.y, cell.z)) - 0.5;
    let jitter_z = unit_float(hash3(seed.split(0xAD90_777D), cell.x, cell.y, cell.z)) - 0.5;
    center + Vec3::new(jitter_x, jitter_y, jitter_z) * jitter
}

#[must_use]
fn distance_2d(metric: WorleyDistanceMetric, delta: Vec2) -> f32 {
    match metric {
        WorleyDistanceMetric::Euclidean => delta.length(),
        WorleyDistanceMetric::Manhattan => delta.abs().x + delta.abs().y,
        WorleyDistanceMetric::Chebyshev => delta.abs().max_element(),
    }
}

#[must_use]
fn distance_3d(metric: WorleyDistanceMetric, delta: Vec3) -> f32 {
    match metric {
        WorleyDistanceMetric::Euclidean => delta.length(),
        WorleyDistanceMetric::Manhattan => delta.abs().x + delta.abs().y + delta.abs().z,
        WorleyDistanceMetric::Chebyshev => delta.abs().max_element(),
    }
}

#[must_use]
fn worley_range(
    metric: WorleyDistanceMetric,
    return_type: WorleyReturnType,
    dimensions: usize,
) -> NoiseRange {
    let conservative_max = match (metric, dimensions) {
        (WorleyDistanceMetric::Euclidean, 2) => 8.0_f32.sqrt(),
        (WorleyDistanceMetric::Euclidean, 3) => 12.0_f32.sqrt(),
        (WorleyDistanceMetric::Manhattan, 2) => 4.0,
        (WorleyDistanceMetric::Manhattan, 3) => 6.0,
        (WorleyDistanceMetric::Chebyshev, 2) => 2.0,
        (WorleyDistanceMetric::Chebyshev, 3) => 2.0,
        _ => 1.0,
    };
    let max = match return_type {
        WorleyReturnType::F1 | WorleyReturnType::F2 | WorleyReturnType::F2MinusF1 => {
            conservative_max
        }
    };
    NoiseRange::new(0.0, max, RangeSemantics::Conservative)
}
