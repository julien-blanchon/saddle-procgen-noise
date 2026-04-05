use bevy::prelude::*;

use crate::{
    config::NoiseRecipe2,
    grid::{Grid2, GridRequest2},
    sample::NoiseSourceExt,
};

/// Stamps a noise pattern onto a mutable buffer at a given position and radius.
///
/// The stamp blends the noise value with the existing buffer value using the provided
/// blend strength (0..1). The noise is sampled in the local stamp coordinate space.
///
/// - `buffer`: mutable height/density values in row-major order
/// - `buffer_size`: dimensions of the buffer (width, height)
/// - `center`: stamp center in buffer pixel coordinates
/// - `radius`: stamp radius in pixels
/// - `strength`: blend factor (0 = no effect, 1 = full replace)
/// - `recipe`: noise recipe to sample
/// - `additive`: if true, adds noise to existing values; if false, blends toward noise value
pub fn stamp_noise(
    buffer: &mut [f32],
    buffer_size: UVec2,
    center: Vec2,
    radius: f32,
    strength: f32,
    recipe: &NoiseRecipe2,
    additive: bool,
) {
    let min_x = ((center.x - radius).floor() as i32).max(0) as u32;
    let max_x = ((center.x + radius).ceil() as i32).min(buffer_size.x as i32 - 1) as u32;
    let min_y = ((center.y - radius).floor() as i32).max(0) as u32;
    let max_y = ((center.y + radius).ceil() as i32).min(buffer_size.y as i32 - 1) as u32;

    let radius_sq = radius * radius;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - center.x;
            let dy = y as f32 - center.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq > radius_sq {
                continue;
            }

            let falloff = 1.0 - (dist_sq / radius_sq).sqrt();
            let falloff = falloff * falloff; // quadratic falloff

            let sample_pos = Vec2::new(dx / radius, dy / radius);
            let noise_val = recipe.sample_normalized(sample_pos);

            let idx = (y * buffer_size.x + x) as usize;
            if idx < buffer.len() {
                let blend = strength * falloff;
                if additive {
                    buffer[idx] += noise_val * blend;
                } else {
                    buffer[idx] = buffer[idx] * (1.0 - blend) + noise_val * blend;
                }
            }
        }
    }
}

/// Applies a noise pattern as an erosion/displacement effect on a grid.
///
/// Multiplies each grid cell by a noise-derived factor, useful for creating
/// erosion patterns, weathering effects, or density modulation.
pub fn modulate_grid(grid: &mut Grid2, recipe: &NoiseRecipe2, grid_request: &GridRequest2) {
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            let point = grid_request.space.sample_position(x, y, grid.size);
            let factor = recipe.sample_normalized(point);
            let idx = (y * grid.size.x + x) as usize;
            if idx < grid.values.len() {
                grid.values[idx] *= factor;
            }
        }
    }
}

/// Creates a binary mask from a noise recipe at a given threshold.
///
/// Returns a `Vec<bool>` where `true` means the noise value exceeded the threshold.
#[must_use]
pub fn noise_mask(recipe: &NoiseRecipe2, grid_request: &GridRequest2, threshold: f32) -> Vec<bool> {
    let mut mask = Vec::with_capacity((grid_request.size.x * grid_request.size.y) as usize);
    for y in 0..grid_request.size.y {
        for x in 0..grid_request.size.x {
            let point = grid_request.space.sample_position(x, y, grid_request.size);
            let value = recipe.sample_normalized(point);
            mask.push(value >= threshold);
        }
    }
    mask
}
