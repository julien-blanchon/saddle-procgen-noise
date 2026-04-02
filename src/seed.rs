use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect, Resource)]
#[reflect(Resource)]
pub struct NoiseSeed(pub u32);

impl NoiseSeed {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn split(self, salt: u32) -> Self {
        Self(self.0 ^ salt.rotate_left(13).wrapping_mul(0x9E37_79B9))
    }
}

impl From<u32> for NoiseSeed {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
