use bevy::prelude::*;

use crate::NoiseSeed;
use crate::config::{
    FractalConfig, NoiseRecipe2, NoiseRecipe4, PerlinConfig, RidgedConfig, SimplexConfig,
    TileConfig, ValueConfig, WarpConfig2, WorleyConfig, WorleyDistanceMetric, WorleyReturnType,
};

/// Fluent builder for constructing `NoiseRecipe2` pipelines.
///
/// ```rust
/// use saddle_procgen_noise::NoiseBuilder;
///
/// let recipe = NoiseBuilder::perlin()
///     .seed(42)
///     .fbm()
///     .octaves(6)
///     .frequency(1.5)
///     .build();
/// ```
pub struct NoiseBuilder {
    recipe: NoiseRecipe2,
}

impl NoiseBuilder {
    // ── Primitive constructors ──

    #[must_use]
    pub fn perlin() -> Self {
        Self {
            recipe: NoiseRecipe2::Perlin(PerlinConfig::default()),
        }
    }

    #[must_use]
    pub fn simplex() -> Self {
        Self {
            recipe: NoiseRecipe2::Simplex(SimplexConfig::default()),
        }
    }

    #[must_use]
    pub fn value() -> Self {
        Self {
            recipe: NoiseRecipe2::Value(ValueConfig::default()),
        }
    }

    #[must_use]
    pub fn worley() -> Self {
        Self {
            recipe: NoiseRecipe2::Worley(WorleyConfig::default()),
        }
    }

    #[must_use]
    pub fn from_recipe(recipe: NoiseRecipe2) -> Self {
        Self { recipe }
    }

    // ── Seed modifier ──

    #[must_use]
    pub fn seed(mut self, seed: u32) -> Self {
        match &mut self.recipe {
            NoiseRecipe2::Perlin(config) => config.seed = NoiseSeed(seed),
            NoiseRecipe2::Simplex(config) => config.seed = NoiseSeed(seed),
            NoiseRecipe2::Value(config) => config.seed = NoiseSeed(seed),
            NoiseRecipe2::Worley(config) => config.seed = NoiseSeed(seed),
            _ => {}
        }
        self
    }

    // ── Worley modifiers ──

    #[must_use]
    pub fn jitter(mut self, jitter: f32) -> Self {
        if let NoiseRecipe2::Worley(config) = &mut self.recipe {
            config.jitter = jitter;
        }
        self
    }

    #[must_use]
    pub fn distance_metric(mut self, metric: WorleyDistanceMetric) -> Self {
        if let NoiseRecipe2::Worley(config) = &mut self.recipe {
            config.distance = metric;
        }
        self
    }

    #[must_use]
    pub fn return_type(mut self, return_type: WorleyReturnType) -> Self {
        if let NoiseRecipe2::Worley(config) = &mut self.recipe {
            config.return_type = return_type;
        }
        self
    }

    // ── Fractal wrappers ──

    #[must_use]
    pub fn fbm(self) -> FractalBuilder {
        FractalBuilder {
            source: self.recipe,
            config: FractalConfig::default(),
            kind: FractalKind::Fbm,
        }
    }

    #[must_use]
    pub fn billow(self) -> FractalBuilder {
        FractalBuilder {
            source: self.recipe,
            config: FractalConfig::default(),
            kind: FractalKind::Billow,
        }
    }

    #[must_use]
    pub fn ridged(self) -> RidgedBuilder {
        RidgedBuilder {
            source: self.recipe,
            config: RidgedConfig::default(),
        }
    }

    // ── Domain warping ──

    #[must_use]
    pub fn warp(self) -> WarpBuilder {
        WarpBuilder {
            base: self.recipe,
            warp_x: NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(101),
            }),
            warp_y: NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(202),
            }),
            config: WarpConfig2::default(),
        }
    }

    // ── Tiling ──

    #[must_use]
    pub fn tiled(self, period: Vec2) -> NoiseBuilder4 {
        let source4 = match self.recipe {
            NoiseRecipe2::Perlin(config) => NoiseRecipe4::Perlin(config),
            NoiseRecipe2::Simplex(config) => NoiseRecipe4::Simplex(config),
            NoiseRecipe2::Value(config) => NoiseRecipe4::Value(config),
            _ => NoiseRecipe4::Perlin(PerlinConfig::default()),
        };
        NoiseBuilder4 {
            recipe: NoiseRecipe2::Tiled {
                source: Box::new(source4),
                config: TileConfig { period },
            },
        }
    }

    // ── Terminal ──

    #[must_use]
    pub fn build(self) -> NoiseRecipe2 {
        self.recipe
    }
}

enum FractalKind {
    Fbm,
    Billow,
}

/// Builder for FBM and Billow fractals.
pub struct FractalBuilder {
    source: NoiseRecipe2,
    config: FractalConfig,
    kind: FractalKind,
}

impl FractalBuilder {
    #[must_use]
    pub fn octaves(mut self, octaves: u8) -> Self {
        self.config.octaves = octaves;
        self
    }

    #[must_use]
    pub fn frequency(mut self, frequency: f32) -> Self {
        self.config.base_frequency = frequency;
        self
    }

    #[must_use]
    pub fn lacunarity(mut self, lacunarity: f32) -> Self {
        self.config.lacunarity = lacunarity;
        self
    }

    #[must_use]
    pub fn gain(mut self, gain: f32) -> Self {
        self.config.gain = gain;
        self
    }

    #[must_use]
    pub fn amplitude(mut self, amplitude: f32) -> Self {
        self.config.amplitude = amplitude;
        self
    }

    /// Wrap the fractal with domain warping.
    #[must_use]
    pub fn warp(self) -> WarpBuilder {
        let recipe = self.build();
        WarpBuilder {
            base: recipe,
            warp_x: NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(101),
            }),
            warp_y: NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(202),
            }),
            config: WarpConfig2::default(),
        }
    }

    #[must_use]
    pub fn build(self) -> NoiseRecipe2 {
        match self.kind {
            FractalKind::Fbm => NoiseRecipe2::Fbm {
                source: Box::new(self.source),
                config: self.config,
            },
            FractalKind::Billow => NoiseRecipe2::Billow {
                source: Box::new(self.source),
                config: self.config,
            },
        }
    }
}

/// Builder for ridged multifractal noise.
pub struct RidgedBuilder {
    source: NoiseRecipe2,
    config: RidgedConfig,
}

impl RidgedBuilder {
    #[must_use]
    pub fn octaves(mut self, octaves: u8) -> Self {
        self.config.fractal.octaves = octaves;
        self
    }

    #[must_use]
    pub fn frequency(mut self, frequency: f32) -> Self {
        self.config.fractal.base_frequency = frequency;
        self
    }

    #[must_use]
    pub fn lacunarity(mut self, lacunarity: f32) -> Self {
        self.config.fractal.lacunarity = lacunarity;
        self
    }

    #[must_use]
    pub fn gain(mut self, gain: f32) -> Self {
        self.config.fractal.gain = gain;
        self
    }

    #[must_use]
    pub fn ridge_offset(mut self, offset: f32) -> Self {
        self.config.ridge_offset = offset;
        self
    }

    #[must_use]
    pub fn weight_strength(mut self, strength: f32) -> Self {
        self.config.weight_strength = strength;
        self
    }

    /// Wrap the ridged fractal with domain warping.
    #[must_use]
    pub fn warp(self) -> WarpBuilder {
        let recipe = self.build();
        WarpBuilder {
            base: recipe,
            warp_x: NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(101),
            }),
            warp_y: NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(202),
            }),
            config: WarpConfig2::default(),
        }
    }

    #[must_use]
    pub fn build(self) -> NoiseRecipe2 {
        NoiseRecipe2::Ridged {
            source: Box::new(self.source),
            config: self.config,
        }
    }
}

/// Builder for domain warping.
pub struct WarpBuilder {
    base: NoiseRecipe2,
    warp_x: NoiseRecipe2,
    warp_y: NoiseRecipe2,
    config: WarpConfig2,
}

impl WarpBuilder {
    #[must_use]
    pub fn warp_amplitude(mut self, amplitude: Vec2) -> Self {
        self.config.amplitude = amplitude;
        self
    }

    #[must_use]
    pub fn warp_frequency(mut self, frequency: f32) -> Self {
        self.config.frequency = frequency;
        self
    }

    #[must_use]
    pub fn warp_sources(mut self, warp_x: NoiseRecipe2, warp_y: NoiseRecipe2) -> Self {
        self.warp_x = warp_x;
        self.warp_y = warp_y;
        self
    }

    #[must_use]
    pub fn build(self) -> NoiseRecipe2 {
        NoiseRecipe2::Warp {
            base: Box::new(self.base),
            warp_x: Box::new(self.warp_x),
            warp_y: Box::new(self.warp_y),
            config: self.config,
        }
    }
}

/// Builder for tiled (seamless) noise via 4D torus mapping.
pub struct NoiseBuilder4 {
    recipe: NoiseRecipe2,
}

impl NoiseBuilder4 {
    #[must_use]
    pub fn build(self) -> NoiseRecipe2 {
        self.recipe
    }
}
