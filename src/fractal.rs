use std::ops::Mul;

use crate::{
    config::{FractalConfig, RidgedConfig},
    sample::{NoiseRange, NoiseSource, NoiseSourceExt, RangeSemantics},
};

#[must_use]
pub fn peak_amplitude_sum(config: FractalConfig) -> f32 {
    let octaves = config.octaves.max(1);
    let mut amplitude = config.amplitude;
    let mut total = 0.0;
    for _ in 0..octaves {
        total += amplitude.abs();
        amplitude *= config.gain;
    }
    total
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fbm<S> {
    pub source: S,
    pub config: FractalConfig,
}

impl<S> Fbm<S> {
    #[must_use]
    pub const fn new(source: S, config: FractalConfig) -> Self {
        Self { source, config }
    }
}

impl<S, I> NoiseSource<I> for Fbm<S>
where
    I: Copy + Mul<f32, Output = I>,
    S: NoiseSource<I>,
{
    fn sample(&self, point: I) -> f32 {
        let mut total = 0.0;
        let mut amplitude = self.config.amplitude;
        let mut frequency = self.config.base_frequency;
        for _ in 0..self.config.octaves.max(1) {
            total += self.source.sample(point * frequency) * amplitude;
            frequency *= self.config.lacunarity;
            amplitude *= self.config.gain;
        }
        total
    }

    fn native_range(&self) -> NoiseRange {
        let base = self.source.native_range();
        let peak = peak_amplitude_sum(self.config);
        NoiseRange::new(base.min * peak, base.max * peak, base.semantics)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Billow<S> {
    pub source: S,
    pub config: FractalConfig,
}

impl<S> Billow<S> {
    #[must_use]
    pub const fn new(source: S, config: FractalConfig) -> Self {
        Self { source, config }
    }
}

impl<S, I> NoiseSource<I> for Billow<S>
where
    I: Copy + Mul<f32, Output = I>,
    S: NoiseSource<I>,
{
    fn sample(&self, point: I) -> f32 {
        let mut total = 0.0;
        let mut amplitude = self.config.amplitude;
        let mut frequency = self.config.base_frequency;
        for _ in 0..self.config.octaves.max(1) {
            let signal = 2.0
                * self
                    .source
                    .sample_signed_normalized(point * frequency)
                    .abs()
                - 1.0;
            total += signal * amplitude;
            frequency *= self.config.lacunarity;
            amplitude *= self.config.gain;
        }
        total
    }

    fn native_range(&self) -> NoiseRange {
        let peak = peak_amplitude_sum(self.config);
        NoiseRange::new(-peak, peak, RangeSemantics::Approximate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ridged<S> {
    pub source: S,
    pub config: RidgedConfig,
}

impl<S> Ridged<S> {
    #[must_use]
    pub const fn new(source: S, config: RidgedConfig) -> Self {
        Self { source, config }
    }
}

impl<S, I> NoiseSource<I> for Ridged<S>
where
    I: Copy + Mul<f32, Output = I>,
    S: NoiseSource<I>,
{
    fn sample(&self, point: I) -> f32 {
        let mut total = 0.0;
        let mut amplitude = self.config.fractal.amplitude;
        let mut frequency = self.config.fractal.base_frequency;
        let mut weight = 1.0;

        for _ in 0..self.config.fractal.octaves.max(1) {
            let sample = self.source.sample_signed_normalized(point * frequency);
            let mut signal = self.config.ridge_offset - sample.abs();
            signal = signal.max(0.0);
            signal *= signal;
            signal *= weight;
            weight = (signal * self.config.weight_strength).clamp(0.0, 1.0);
            total += signal * amplitude;
            frequency *= self.config.fractal.lacunarity;
            amplitude *= self.config.fractal.gain;
        }

        total
    }

    fn native_range(&self) -> NoiseRange {
        let peak = peak_amplitude_sum(self.config.fractal);
        NoiseRange::new(0.0, peak, RangeSemantics::Approximate)
    }
}
