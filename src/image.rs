use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use serde::{Deserialize, Serialize};

use crate::{grid::Grid2, sample::NoiseRange};

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct GradientRamp {
    pub stops: Vec<GradientStop>,
}

impl GradientRamp {
    #[must_use]
    pub fn grayscale() -> Self {
        Self {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: Color::srgb(0.0, 0.0, 0.0),
                },
                GradientStop {
                    position: 1.0,
                    color: Color::srgb(1.0, 1.0, 1.0),
                },
            ],
        }
    }

    #[must_use]
    pub fn terrain() -> Self {
        Self {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: Color::srgb(0.03, 0.06, 0.14),
                },
                GradientStop {
                    position: 0.28,
                    color: Color::srgb(0.10, 0.32, 0.60),
                },
                GradientStop {
                    position: 0.40,
                    color: Color::srgb(0.18, 0.52, 0.34),
                },
                GradientStop {
                    position: 0.60,
                    color: Color::srgb(0.56, 0.50, 0.30),
                },
                GradientStop {
                    position: 0.82,
                    color: Color::srgb(0.78, 0.76, 0.68),
                },
                GradientStop {
                    position: 1.0,
                    color: Color::srgb(0.96, 0.96, 0.98),
                },
            ],
        }
    }

    #[must_use]
    pub fn heatmap() -> Self {
        Self {
            stops: vec![
                GradientStop {
                    position: 0.0,
                    color: Color::srgb(0.07, 0.02, 0.14),
                },
                GradientStop {
                    position: 0.25,
                    color: Color::srgb(0.17, 0.13, 0.53),
                },
                GradientStop {
                    position: 0.5,
                    color: Color::srgb(0.75, 0.22, 0.26),
                },
                GradientStop {
                    position: 0.75,
                    color: Color::srgb(0.96, 0.65, 0.20),
                },
                GradientStop {
                    position: 1.0,
                    color: Color::srgb(0.98, 0.96, 0.78),
                },
            ],
        }
    }

    #[must_use]
    pub fn sample(&self, t: f32) -> [u8; 4] {
        let t = t.clamp(0.0, 1.0);
        if self.stops.is_empty() {
            return [0, 0, 0, 255];
        }
        if t <= self.stops[0].position {
            return color_to_rgba(self.stops[0].color);
        }
        if t >= self.stops[self.stops.len() - 1].position {
            return color_to_rgba(self.stops[self.stops.len() - 1].color);
        }

        for window in self.stops.windows(2) {
            let start = &window[0];
            let end = &window[1];
            if (start.position..=end.position).contains(&t) {
                let span = (end.position - start.position).max(f32::EPSILON);
                let local_t = (t - start.position) / span;
                let a = start.color.to_srgba();
                let b = end.color.to_srgba();
                return [
                    ((a.red + (b.red - a.red) * local_t) * 255.0).round() as u8,
                    ((a.green + (b.green - a.green) * local_t) * 255.0).round() as u8,
                    ((a.blue + (b.blue - a.blue) * local_t) * 255.0).round() as u8,
                    ((a.alpha + (b.alpha - a.alpha) * local_t) * 255.0).round() as u8,
                ];
            }
        }

        [0, 0, 0, 255]
    }
}

impl Default for GradientRamp {
    fn default() -> Self {
        Self::terrain()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default, Serialize, Deserialize)]
pub enum ImageOutputMode {
    #[default]
    Grayscale,
    Gradient,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Default, Serialize, Deserialize)]
pub enum ImageNormalization {
    Observed,
    #[default]
    Conservative,
    Explicit(Vec2),
}

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct NoiseImageSettings {
    pub mode: ImageOutputMode,
    pub normalization: ImageNormalization,
    pub gradient: GradientRamp,
}

impl Default for NoiseImageSettings {
    fn default() -> Self {
        Self {
            mode: ImageOutputMode::Gradient,
            normalization: ImageNormalization::Conservative,
            gradient: GradientRamp::terrain(),
        }
    }
}

#[must_use]
pub fn grid_to_grayscale_image(grid: &Grid2, range: Option<NoiseRange>) -> Image {
    let normalization = range.unwrap_or(NoiseRange::new(
        grid.stats.min,
        grid.stats.max,
        crate::RangeSemantics::Approximate,
    ));
    let mut bytes = Vec::with_capacity(grid.values.len() * 4);
    for value in &grid.values {
        let v = (normalization.normalize_clamped(*value) * 255.0).round() as u8;
        bytes.extend_from_slice(&[v, v, v, 255]);
    }
    Image::new(
        Extent3d {
            width: grid.size.x,
            height: grid.size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

#[must_use]
pub fn grid_to_gradient_image(
    grid: &Grid2,
    settings: &NoiseImageSettings,
    range: Option<NoiseRange>,
) -> Image {
    let normalization = match settings.normalization {
        ImageNormalization::Observed => NoiseRange::new(
            grid.stats.min,
            grid.stats.max,
            crate::RangeSemantics::Approximate,
        ),
        ImageNormalization::Conservative => range.unwrap_or(NoiseRange::new(
            grid.stats.min,
            grid.stats.max,
            crate::RangeSemantics::Approximate,
        )),
        ImageNormalization::Explicit(explicit) => {
            NoiseRange::new(explicit.x, explicit.y, crate::RangeSemantics::Strict)
        }
    };

    let mut bytes = Vec::with_capacity(grid.values.len() * 4);
    for value in &grid.values {
        let t = normalization.normalize_clamped(*value);
        match settings.mode {
            ImageOutputMode::Grayscale => {
                let v = (t * 255.0).round() as u8;
                bytes.extend_from_slice(&[v, v, v, 255]);
            }
            ImageOutputMode::Gradient => bytes.extend_from_slice(&settings.gradient.sample(t)),
        }
    }

    Image::new(
        Extent3d {
            width: grid.size.x,
            height: grid.size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

#[must_use]
pub fn pack_scalar_layers_rgba(
    r: &Grid2,
    g: &Grid2,
    b: &Grid2,
    a: Option<&Grid2>,
    normalization: [NoiseRange; 4],
) -> Image {
    let mut bytes = Vec::with_capacity(r.values.len() * 4);
    for index in 0..r.values.len() {
        let rv = (normalization[0].normalize_clamped(r.values[index]) * 255.0).round() as u8;
        let gv = (normalization[1].normalize_clamped(g.values[index]) * 255.0).round() as u8;
        let bv = (normalization[2].normalize_clamped(b.values[index]) * 255.0).round() as u8;
        let av = a
            .map(|grid| {
                (normalization[3].normalize_clamped(grid.values[index]) * 255.0).round() as u8
            })
            .unwrap_or(255);
        bytes.extend_from_slice(&[rv, gv, bv, av]);
    }

    Image::new(
        Extent3d {
            width: r.size.x,
            height: r.size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

#[inline]
fn color_to_rgba(color: Color) -> [u8; 4] {
    let color = color.to_srgba();
    [
        (color.red * 255.0).round() as u8,
        (color.green * 255.0).round() as u8,
        (color.blue * 255.0).round() as u8,
        (color.alpha * 255.0).round() as u8,
    ]
}
