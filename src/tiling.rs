use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::{
    config::TileConfig,
    sample::{NoiseRange, NoiseSource},
};

#[must_use]
pub fn map_to_torus_4d(point: Vec2, config: TileConfig) -> Vec4 {
    let period = Vec2::new(
        config.period.x.max(f32::EPSILON),
        config.period.y.max(f32::EPSILON),
    );
    let ux = point.x / period.x;
    let uy = point.y / period.y;
    let angle_x = ux * TAU;
    let angle_y = uy * TAU;
    let radius_x = period.x / TAU;
    let radius_y = period.y / TAU;
    Vec4::new(
        angle_x.cos() * radius_x,
        angle_x.sin() * radius_x,
        angle_y.cos() * radius_y,
        angle_y.sin() * radius_y,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tiled2<S> {
    pub source: S,
    pub config: TileConfig,
}

impl<S> Tiled2<S> {
    #[must_use]
    pub const fn new(source: S, config: TileConfig) -> Self {
        Self { source, config }
    }
}

impl<S> NoiseSource<Vec2> for Tiled2<S>
where
    S: NoiseSource<Vec4>,
{
    fn sample(&self, point: Vec2) -> f32 {
        self.source.sample(map_to_torus_4d(point, self.config))
    }

    fn native_range(&self) -> NoiseRange {
        self.source.native_range()
    }
}
