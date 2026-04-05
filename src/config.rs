use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::NoiseSeed;

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct PerlinConfig {
    pub seed: NoiseSeed,
}

impl Default for PerlinConfig {
    fn default() -> Self {
        Self {
            seed: NoiseSeed::new(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SimplexConfig {
    pub seed: NoiseSeed,
}

impl Default for SimplexConfig {
    fn default() -> Self {
        Self {
            seed: NoiseSeed::new(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct ValueConfig {
    pub seed: NoiseSeed,
}

impl Default for ValueConfig {
    fn default() -> Self {
        Self {
            seed: NoiseSeed::new(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default, Serialize, Deserialize)]
pub enum WorleyDistanceMetric {
    #[default]
    Euclidean,
    Manhattan,
    Chebyshev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default, Serialize, Deserialize)]
pub enum WorleyReturnType {
    #[default]
    F1,
    F2,
    F2MinusF1,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct WorleyConfig {
    pub seed: NoiseSeed,
    pub jitter: f32,
    pub distance: WorleyDistanceMetric,
    pub return_type: WorleyReturnType,
}

impl Default for WorleyConfig {
    fn default() -> Self {
        Self {
            seed: NoiseSeed::new(0),
            jitter: 1.0,
            distance: WorleyDistanceMetric::Euclidean,
            return_type: WorleyReturnType::F1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct FractalConfig {
    pub octaves: u8,
    pub base_frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
    pub amplitude: f32,
}

impl Default for FractalConfig {
    fn default() -> Self {
        Self {
            octaves: 5,
            base_frequency: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
            amplitude: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct RidgedConfig {
    pub fractal: FractalConfig,
    pub ridge_offset: f32,
    pub weight_strength: f32,
}

impl Default for RidgedConfig {
    fn default() -> Self {
        Self {
            fractal: FractalConfig::default(),
            ridge_offset: 1.0,
            weight_strength: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct WarpConfig2 {
    pub amplitude: Vec2,
    pub frequency: f32,
    pub offset_x: Vec2,
    pub offset_y: Vec2,
}

impl Default for WarpConfig2 {
    fn default() -> Self {
        Self {
            amplitude: Vec2::splat(0.75),
            frequency: 1.0,
            offset_x: Vec2::new(17.3, 9.1),
            offset_y: Vec2::new(-11.7, 23.9),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct WarpConfig3 {
    pub amplitude: Vec3,
    pub frequency: f32,
    pub offset_x: Vec3,
    pub offset_y: Vec3,
    pub offset_z: Vec3,
}

impl Default for WarpConfig3 {
    fn default() -> Self {
        Self {
            amplitude: Vec3::splat(0.75),
            frequency: 1.0,
            offset_x: Vec3::new(17.3, 9.1, 3.7),
            offset_y: Vec3::new(-11.7, 23.9, 12.4),
            offset_z: Vec3::new(4.2, -19.3, 27.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct TileConfig {
    pub period: Vec2,
}

impl Default for TileConfig {
    fn default() -> Self {
        Self { period: Vec2::ONE }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DomainTransform2 {
    pub translation: Vec2,
    pub scale: Vec2,
    pub rotation_radians: f32,
}

impl DomainTransform2 {
    #[must_use]
    pub fn apply(self, point: Vec2) -> Vec2 {
        let scaled = point * self.scale;
        let rotated = Mat2::from_angle(self.rotation_radians) * scaled;
        rotated + self.translation
    }
}

impl Default for DomainTransform2 {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation_radians: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DomainTransform3 {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl DomainTransform3 {
    #[must_use]
    pub fn apply(self, point: Vec3) -> Vec3 {
        self.rotation * (point * self.scale) + self.translation
    }
}

impl Default for DomainTransform3 {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct DomainTransform4 {
    pub translation: Vec4,
    pub scale: Vec4,
}

impl DomainTransform4 {
    #[must_use]
    pub fn apply(self, point: Vec4) -> Vec4 {
        point * self.scale + self.translation
    }
}

impl Default for DomainTransform4 {
    fn default() -> Self {
        Self {
            translation: Vec4::ZERO,
            scale: Vec4::ONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct GridSpace2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl GridSpace2 {
    #[must_use]
    pub fn sample_position(self, x: u32, y: u32, size: UVec2) -> Vec2 {
        let u = if size.x <= 1 {
            0.0
        } else {
            x as f32 / (size.x - 1) as f32
        };
        let v = if size.y <= 1 {
            0.0
        } else {
            y as f32 / (size.y - 1) as f32
        };
        Vec2::new(
            self.min.x + (self.max.x - self.min.x) * u,
            self.min.y + (self.max.y - self.min.y) * v,
        )
    }
}

impl Default for GridSpace2 {
    fn default() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct GridSpace3 {
    pub min: Vec3,
    pub max: Vec3,
}

impl GridSpace3 {
    #[must_use]
    pub fn sample_position(self, x: u32, y: u32, z: u32, size: UVec3) -> Vec3 {
        let u = if size.x <= 1 {
            0.0
        } else {
            x as f32 / (size.x - 1) as f32
        };
        let v = if size.y <= 1 {
            0.0
        } else {
            y as f32 / (size.y - 1) as f32
        };
        let w = if size.z <= 1 {
            0.0
        } else {
            z as f32 / (size.z - 1) as f32
        };
        Vec3::new(
            self.min.x + (self.max.x - self.min.x) * u,
            self.min.y + (self.max.y - self.min.y) * v,
            self.min.z + (self.max.z - self.min.z) * w,
        )
    }
}

impl Default for GridSpace3 {
    fn default() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(no_field_bounds)]
pub enum NoiseRecipe4 {
    Perlin(PerlinConfig),
    Simplex(SimplexConfig),
    Value(ValueConfig),
    Fbm {
        #[reflect(ignore)]
        source: Box<NoiseRecipe4>,
        config: FractalConfig,
    },
    Billow {
        #[reflect(ignore)]
        source: Box<NoiseRecipe4>,
        config: FractalConfig,
    },
    Ridged {
        #[reflect(ignore)]
        source: Box<NoiseRecipe4>,
        config: RidgedConfig,
    },
    Transformed {
        #[reflect(ignore)]
        source: Box<NoiseRecipe4>,
        transform: DomainTransform4,
    },
}

impl NoiseRecipe4 {
    #[must_use]
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Perlin(_) => "Perlin4D",
            Self::Simplex(_) => "Simplex4D",
            Self::Value(_) => "Value4D",
            Self::Fbm { .. } => "Fbm4D",
            Self::Billow { .. } => "Billow4D",
            Self::Ridged { .. } => "Ridged4D",
            Self::Transformed { .. } => "Transformed4D",
        }
    }

    #[must_use]
    pub fn debug_stack(&self) -> String {
        match self {
            Self::Perlin(config) => format!("Perlin4D(seed={})", config.seed.0),
            Self::Simplex(config) => format!("Simplex4D(seed={})", config.seed.0),
            Self::Value(config) => format!("Value4D(seed={})", config.seed.0),
            Self::Fbm { source, config } => {
                format!(
                    "Fbm4D(octaves={}, source={})",
                    config.octaves,
                    source.debug_stack()
                )
            }
            Self::Billow { source, config } => format!(
                "Billow4D(octaves={}, source={})",
                config.octaves,
                source.debug_stack()
            ),
            Self::Ridged { source, config } => format!(
                "Ridged4D(octaves={}, source={})",
                config.fractal.octaves,
                source.debug_stack()
            ),
            Self::Transformed { source, .. } => {
                format!("Transformed4D(source={})", source.debug_stack())
            }
        }
    }
}

impl Default for NoiseRecipe4 {
    fn default() -> Self {
        Self::Perlin(PerlinConfig::default())
    }
}

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(no_field_bounds)]
pub enum NoiseRecipe2 {
    Perlin(PerlinConfig),
    Simplex(SimplexConfig),
    Value(ValueConfig),
    Worley(WorleyConfig),
    Fbm {
        #[reflect(ignore)]
        source: Box<NoiseRecipe2>,
        config: FractalConfig,
    },
    Billow {
        #[reflect(ignore)]
        source: Box<NoiseRecipe2>,
        config: FractalConfig,
    },
    Ridged {
        #[reflect(ignore)]
        source: Box<NoiseRecipe2>,
        config: RidgedConfig,
    },
    Warp {
        #[reflect(ignore)]
        base: Box<NoiseRecipe2>,
        #[reflect(ignore)]
        warp_x: Box<NoiseRecipe2>,
        #[reflect(ignore)]
        warp_y: Box<NoiseRecipe2>,
        config: WarpConfig2,
    },
    Transformed {
        #[reflect(ignore)]
        source: Box<NoiseRecipe2>,
        transform: DomainTransform2,
    },
    Tiled {
        #[reflect(ignore)]
        source: Box<NoiseRecipe4>,
        config: TileConfig,
    },
    /// Multi-level domain warping (Quilez pattern): `f(p + fbm(p + fbm(p)))`.
    ///
    /// Each layer applies domain warping using the result of the previous layer.
    /// The `layers` field contains the warp recipes applied in sequence,
    /// each displacing the coordinate by its sampled value scaled by `amplitude`.
    MultiWarp {
        #[reflect(ignore)]
        base: Box<NoiseRecipe2>,
        #[reflect(ignore)]
        layers: Vec<NoiseRecipe2>,
        amplitude: f32,
    },
}

impl NoiseRecipe2 {
    #[must_use]
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Perlin(_) => "Perlin",
            Self::Simplex(_) => "Simplex",
            Self::Value(_) => "Value",
            Self::Worley(_) => "Worley",
            Self::Fbm { .. } => "Fbm",
            Self::Billow { .. } => "Billow",
            Self::Ridged { .. } => "Ridged",
            Self::Warp { .. } => "Warp",
            Self::Transformed { .. } => "Transformed",
            Self::Tiled { .. } => "Tiled",
            Self::MultiWarp { .. } => "MultiWarp",
        }
    }

    #[must_use]
    pub fn debug_stack(&self) -> String {
        match self {
            Self::Perlin(config) => format!("Perlin(seed={})", config.seed.0),
            Self::Simplex(config) => format!("Simplex(seed={})", config.seed.0),
            Self::Value(config) => format!("Value(seed={})", config.seed.0),
            Self::Worley(config) => format!(
                "Worley(seed={}, return={:?}, metric={:?})",
                config.seed.0, config.return_type, config.distance
            ),
            Self::Fbm { source, config } => {
                format!(
                    "Fbm(octaves={}, source={})",
                    config.octaves,
                    source.debug_stack()
                )
            }
            Self::Billow { source, config } => format!(
                "Billow(octaves={}, source={})",
                config.octaves,
                source.debug_stack()
            ),
            Self::Ridged { source, config } => format!(
                "Ridged(octaves={}, source={})",
                config.fractal.octaves,
                source.debug_stack()
            ),
            Self::Warp {
                base,
                warp_x,
                warp_y,
                ..
            } => format!(
                "Warp(base={}, x={}, y={})",
                base.debug_stack(),
                warp_x.debug_stack(),
                warp_y.debug_stack()
            ),
            Self::Transformed { source, .. } => {
                format!("Transformed(source={})", source.debug_stack())
            }
            Self::Tiled { source, .. } => format!("Tiled(source={})", source.debug_stack()),
            Self::MultiWarp {
                base,
                layers,
                amplitude,
            } => format!(
                "MultiWarp(base={}, layers={}, amp={:.2})",
                base.debug_stack(),
                layers.len(),
                amplitude
            ),
        }
    }
}

impl Default for NoiseRecipe2 {
    fn default() -> Self {
        Self::Perlin(PerlinConfig::default())
    }
}
