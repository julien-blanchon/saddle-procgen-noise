use crate::{
    config::{WarpConfig2, WarpConfig3},
    sample::{NoiseRange, NoiseSource, NoiseSourceExt},
};
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomainWarp2<Base, X, Y> {
    pub base: Base,
    pub warp_x: X,
    pub warp_y: Y,
    pub config: WarpConfig2,
}

impl<Base, X, Y> DomainWarp2<Base, X, Y> {
    #[must_use]
    pub const fn new(base: Base, warp_x: X, warp_y: Y, config: WarpConfig2) -> Self {
        Self {
            base,
            warp_x,
            warp_y,
            config,
        }
    }
}

impl<Base, X, Y> NoiseSource<Vec2> for DomainWarp2<Base, X, Y>
where
    Base: NoiseSource<Vec2>,
    X: NoiseSource<Vec2>,
    Y: NoiseSource<Vec2>,
{
    fn sample(&self, point: Vec2) -> f32 {
        let warp_point = point * self.config.frequency;
        let dx = self
            .warp_x
            .sample_signed_normalized(warp_point + self.config.offset_x)
            * self.config.amplitude.x;
        let dy = self
            .warp_y
            .sample_signed_normalized(warp_point + self.config.offset_y)
            * self.config.amplitude.y;
        self.base.sample(point + Vec2::new(dx, dy))
    }

    fn native_range(&self) -> NoiseRange {
        self.base.native_range()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomainWarp3<Base, X, Y, Z> {
    pub base: Base,
    pub warp_x: X,
    pub warp_y: Y,
    pub warp_z: Z,
    pub config: WarpConfig3,
}

impl<Base, X, Y, Z> DomainWarp3<Base, X, Y, Z> {
    #[must_use]
    pub const fn new(base: Base, warp_x: X, warp_y: Y, warp_z: Z, config: WarpConfig3) -> Self {
        Self {
            base,
            warp_x,
            warp_y,
            warp_z,
            config,
        }
    }
}

impl<Base, X, Y, Z> NoiseSource<Vec3> for DomainWarp3<Base, X, Y, Z>
where
    Base: NoiseSource<Vec3>,
    X: NoiseSource<Vec3>,
    Y: NoiseSource<Vec3>,
    Z: NoiseSource<Vec3>,
{
    fn sample(&self, point: Vec3) -> f32 {
        let warp_point = point * self.config.frequency;
        let dx = self
            .warp_x
            .sample_signed_normalized(warp_point + self.config.offset_x)
            * self.config.amplitude.x;
        let dy = self
            .warp_y
            .sample_signed_normalized(warp_point + self.config.offset_y)
            * self.config.amplitude.y;
        let dz = self
            .warp_z
            .sample_signed_normalized(warp_point + self.config.offset_z)
            * self.config.amplitude.z;
        self.base.sample(point + Vec3::new(dx, dy, dz))
    }

    fn native_range(&self) -> NoiseRange {
        self.base.native_range()
    }
}
