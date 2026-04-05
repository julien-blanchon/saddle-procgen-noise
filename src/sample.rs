use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum RangeSemantics {
    Strict,
    Approximate,
    Conservative,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct NoiseRange {
    pub min: f32,
    pub max: f32,
    pub semantics: RangeSemantics,
}

impl NoiseRange {
    #[must_use]
    pub const fn new(min: f32, max: f32, semantics: RangeSemantics) -> Self {
        Self {
            min,
            max,
            semantics,
        }
    }

    #[must_use]
    pub fn normalize_clamped(self, value: f32) -> f32 {
        let span = (self.max - self.min).max(f32::EPSILON);
        ((value - self.min) / span).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn signed_clamped(self, value: f32) -> f32 {
        self.normalize_clamped(value) * 2.0 - 1.0
    }

    #[must_use]
    pub fn span(self) -> f32 {
        self.max - self.min
    }
}

pub trait NoiseSource<I>: Send + Sync {
    fn sample(&self, point: I) -> f32;

    fn native_range(&self) -> NoiseRange;
}

impl<I, T> NoiseSource<I> for &T
where
    T: NoiseSource<I> + ?Sized,
{
    fn sample(&self, point: I) -> f32 {
        (*self).sample(point)
    }

    fn native_range(&self) -> NoiseRange {
        (*self).native_range()
    }
}

impl<I, T> NoiseSource<I> for Box<T>
where
    T: NoiseSource<I> + ?Sized,
{
    fn sample(&self, point: I) -> f32 {
        (**self).sample(point)
    }

    fn native_range(&self) -> NoiseRange {
        (**self).native_range()
    }
}

pub trait NoiseSourceExt<I>: NoiseSource<I> {
    fn sample_normalized(&self, point: I) -> f32
    where
        I: Copy,
    {
        self.native_range().normalize_clamped(self.sample(point))
    }

    fn sample_signed_normalized(&self, point: I) -> f32
    where
        I: Copy,
    {
        self.native_range().signed_clamped(self.sample(point))
    }
}

impl<T, I> NoiseSourceExt<I> for T where T: NoiseSource<I> {}
