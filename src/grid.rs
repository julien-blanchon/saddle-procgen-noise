use std::time::Instant;

use bevy::prelude::*;

use crate::{
    config::{GridSpace2, GridSpace3, NoiseRecipe2, NoiseRecipe4},
    fractal::{Billow, Fbm, Ridged, peak_amplitude_sum},
    image::{NoiseImageSettings, grid_to_gradient_image},
    perlin::Perlin,
    sample::{NoiseRange, NoiseSource, RangeSemantics},
    simplex::Simplex,
    tiling::map_to_torus_4d,
    warp::DomainWarp2,
    worley::Worley,
};

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct GridRequest2 {
    pub size: UVec2,
    pub space: GridSpace2,
}

impl Default for GridRequest2 {
    fn default() -> Self {
        Self {
            size: UVec2::splat(256),
            space: GridSpace2 {
                min: Vec2::new(-2.0, -2.0),
                max: Vec2::new(2.0, 2.0),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct GridRequest3 {
    pub size: UVec3,
    pub space: GridSpace3,
}

impl Default for GridRequest3 {
    fn default() -> Self {
        Self {
            size: UVec3::new(48, 48, 32),
            space: GridSpace3 {
                min: Vec3::new(-2.0, -2.0, -2.0),
                max: Vec3::new(2.0, 2.0, 2.0),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct GridStats {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub variance: f32,
}

impl Default for GridStats {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            variance: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Grid2 {
    pub size: UVec2,
    pub values: Vec<f32>,
    pub stats: GridStats,
}

impl Grid2 {
    #[must_use]
    pub fn signature(&self) -> u64 {
        self.values
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, value| {
                (hash ^ value.to_bits() as u64).wrapping_mul(0x100_0000_01b3)
            })
    }

    #[must_use]
    pub fn threshold_mask(&self, threshold: f32) -> MaskGrid2 {
        let values = self
            .values
            .iter()
            .map(|value| u8::from(*value >= threshold))
            .collect();
        MaskGrid2 {
            size: self.size,
            values,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Grid3 {
    pub size: UVec3,
    pub values: Vec<f32>,
    pub stats: GridStats,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct MaskGrid2 {
    pub size: UVec2,
    pub values: Vec<u8>,
}

impl MaskGrid2 {
    #[must_use]
    pub fn coverage_ratio(&self) -> f32 {
        let active = self.values.iter().filter(|value| **value != 0).count();
        active as f32 / self.values.len().max(1) as f32
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct GridSampleRequest {
    pub recipe: NoiseRecipe2,
    pub grid: GridRequest2,
    pub image: NoiseImageSettings,
    pub async_generation: bool,
}

impl Default for GridSampleRequest {
    fn default() -> Self {
        Self {
            recipe: NoiseRecipe2::default(),
            grid: GridRequest2::default(),
            image: NoiseImageSettings::default(),
            async_generation: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridSampleResult {
    pub grid: Grid2,
    pub image: Image,
    pub signature: u64,
    pub duration_ms: f32,
}

#[must_use]
pub fn sample_grid2<S>(source: &S, request: &GridRequest2) -> Grid2
where
    S: NoiseSource<Vec2>,
{
    let mut values = Vec::with_capacity((request.size.x * request.size.y) as usize);
    for y in 0..request.size.y {
        for x in 0..request.size.x {
            let point = request.space.sample_position(x, y, request.size);
            values.push(source.sample(point));
        }
    }
    let stats = compute_stats(&values);
    Grid2 {
        size: request.size,
        values,
        stats,
    }
}

#[must_use]
pub fn sample_grid3<S>(source: &S, request: &GridRequest3) -> Grid3
where
    S: NoiseSource<Vec3>,
{
    let mut values =
        Vec::with_capacity((request.size.x * request.size.y * request.size.z) as usize);
    for z in 0..request.size.z {
        for y in 0..request.size.y {
            for x in 0..request.size.x {
                let point = request.space.sample_position(x, y, z, request.size);
                values.push(source.sample(point));
            }
        }
    }
    let stats = compute_stats(&values);
    Grid3 {
        size: request.size,
        values,
        stats,
    }
}

#[must_use]
pub fn generate_grid_sample(request: &GridSampleRequest) -> GridSampleResult {
    let started = Instant::now();
    let grid = sample_grid2(&request.recipe, &request.grid);
    let range = Some(request.recipe.native_range());
    let image = grid_to_gradient_image(&grid, &request.image, range);
    GridSampleResult {
        signature: grid.signature(),
        duration_ms: started.elapsed().as_secs_f32() * 1000.0,
        image,
        grid,
    }
}

impl NoiseSource<Vec2> for NoiseRecipe2 {
    fn sample(&self, point: Vec2) -> f32 {
        match self {
            Self::Perlin(config) => Perlin::from(*config).sample(point),
            Self::Simplex(config) => Simplex::from(*config).sample(point),
            Self::Worley(config) => Worley::from(*config).sample(point),
            Self::Fbm { source, config } => Fbm::new(source.as_ref(), *config).sample(point),
            Self::Billow { source, config } => Billow::new(source.as_ref(), *config).sample(point),
            Self::Ridged { source, config } => Ridged::new(source.as_ref(), *config).sample(point),
            Self::Warp {
                base,
                warp_x,
                warp_y,
                config,
            } => DomainWarp2::new(base.as_ref(), warp_x.as_ref(), warp_y.as_ref(), *config)
                .sample(point),
            Self::Transformed { source, transform } => source.sample(transform.apply(point)),
            Self::Tiled { source, config } => source.sample(map_to_torus_4d(point, *config)),
        }
    }

    fn native_range(&self) -> NoiseRange {
        match self {
            Self::Perlin(_) => NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate),
            Self::Simplex(_) => NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate),
            Self::Worley(config) => {
                <Worley as NoiseSource<Vec2>>::native_range(&Worley::from(*config))
            }
            Self::Fbm { source, config } => {
                let base = source.native_range();
                let peak = peak_amplitude_sum(*config);
                NoiseRange::new(base.min * peak, base.max * peak, base.semantics)
            }
            Self::Billow { config, .. } => {
                let peak = peak_amplitude_sum(*config);
                NoiseRange::new(-peak, peak, RangeSemantics::Approximate)
            }
            Self::Ridged { config, .. } => {
                let peak = peak_amplitude_sum(config.fractal);
                NoiseRange::new(0.0, peak, RangeSemantics::Approximate)
            }
            Self::Warp { base, .. } => base.native_range(),
            Self::Transformed { source, .. } => source.native_range(),
            Self::Tiled { source, .. } => source.native_range(),
        }
    }
}

impl NoiseSource<Vec4> for NoiseRecipe4 {
    fn sample(&self, point: Vec4) -> f32 {
        match self {
            Self::Perlin(config) => Perlin::from(*config).sample(point),
            Self::Simplex(config) => Simplex::from(*config).sample(point),
            Self::Fbm { source, config } => Fbm::new(source.as_ref(), *config).sample(point),
            Self::Billow { source, config } => Billow::new(source.as_ref(), *config).sample(point),
            Self::Ridged { source, config } => Ridged::new(source.as_ref(), *config).sample(point),
            Self::Transformed { source, transform } => source.sample(transform.apply(point)),
        }
    }

    fn native_range(&self) -> NoiseRange {
        match self {
            Self::Perlin(_) => NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate),
            Self::Simplex(_) => NoiseRange::new(-1.0, 1.0, RangeSemantics::Approximate),
            Self::Fbm { source, config } => {
                let base = source.native_range();
                let peak = peak_amplitude_sum(*config);
                NoiseRange::new(base.min * peak, base.max * peak, base.semantics)
            }
            Self::Billow { config, .. } => {
                let peak = peak_amplitude_sum(*config);
                NoiseRange::new(-peak, peak, RangeSemantics::Approximate)
            }
            Self::Ridged { config, .. } => {
                let peak = peak_amplitude_sum(config.fractal);
                NoiseRange::new(0.0, peak, RangeSemantics::Approximate)
            }
            Self::Transformed { source, .. } => source.native_range(),
        }
    }
}

#[must_use]
fn compute_stats(values: &[f32]) -> GridStats {
    if values.is_empty() {
        return GridStats::default();
    }

    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    let mut sum = 0.0;
    for value in values {
        min = min.min(*value);
        max = max.max(*value);
        sum += *value;
    }
    let mean = sum / values.len() as f32;
    let variance = values
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f32>()
        / values.len() as f32;

    GridStats {
        min,
        max,
        mean,
        variance,
    }
}
