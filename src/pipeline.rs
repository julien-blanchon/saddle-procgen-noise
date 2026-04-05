use bevy::prelude::*;

use crate::{
    config::NoiseRecipe2,
    grid::{GridRequest2, GridSampleRequest, generate_grid_sample},
    image::NoiseImageSettings,
    sample::NoiseSource,
};

/// Component that drives automatic noise image generation on an entity.
///
/// When attached to an entity, the pipeline system will generate a noise image
/// and store the result in `NoiseImageOutput`. The image is regenerated whenever
/// this component changes.
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct NoiseImageGenerator {
    pub recipe: NoiseRecipe2,
    pub grid: GridRequest2,
    pub image_settings: NoiseImageSettings,
}

/// Component storing the output of noise generation.
/// Automatically populated by the pipeline system when `NoiseImageGenerator` is present.
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct NoiseImageOutput {
    pub handle: Option<Handle<Image>>,
    pub signature: u64,
    pub duration_ms: f32,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub variance: f32,
}

pub(crate) fn generate_noise_images(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    changed: Query<(Entity, &NoiseImageGenerator), Changed<NoiseImageGenerator>>,
    mut outputs: Query<&mut NoiseImageOutput>,
) {
    for (entity, generator) in changed.iter() {
        let request = GridSampleRequest {
            recipe: generator.recipe.clone(),
            grid: generator.grid.clone(),
            image: generator.image_settings.clone(),
            async_generation: false,
        };

        let result = generate_grid_sample(&request);

        if let Ok(mut output) = outputs.get_mut(entity) {
            if let Some(existing_handle) = output.handle.as_ref() {
                if let Some(existing_image) = images.get_mut(existing_handle) {
                    *existing_image = result.image;
                    output.signature = result.signature;
                    output.duration_ms = result.duration_ms;
                    output.min = result.grid.stats.min;
                    output.max = result.grid.stats.max;
                    output.mean = result.grid.stats.mean;
                    output.variance = result.grid.stats.variance;
                    continue;
                }
            }
            let handle = images.add(result.image);
            output.handle = Some(handle);
            output.signature = result.signature;
            output.duration_ms = result.duration_ms;
            output.min = result.grid.stats.min;
            output.max = result.grid.stats.max;
            output.mean = result.grid.stats.mean;
            output.variance = result.grid.stats.variance;
        } else {
            let handle = images.add(result.image);
            commands.entity(entity).insert(NoiseImageOutput {
                handle: Some(handle),
                signature: result.signature,
                duration_ms: result.duration_ms,
                min: result.grid.stats.min,
                max: result.grid.stats.max,
                mean: result.grid.stats.mean,
                variance: result.grid.stats.variance,
            });
        }
    }
}

/// Generates a heightmap as a `Vec<f32>` buffer from a noise recipe.
///
/// Returns a flat buffer of height values in row-major order, suitable for
/// terrain mesh generation. Values are normalized to `[0, 1]`.
#[must_use]
pub fn generate_heightmap(recipe: &NoiseRecipe2, grid: &GridRequest2) -> Vec<f32> {
    let range = recipe.native_range();
    let grid_data = crate::sample_grid2(recipe, grid);
    grid_data
        .values
        .iter()
        .map(|v| range.normalize_clamped(*v))
        .collect()
}
