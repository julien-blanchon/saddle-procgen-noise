#![doc = include_str!("../README.md")]

mod components;
mod config;
mod fractal;
mod grid;
mod hash;
mod image;
mod perlin;
mod remap;
mod sample;
mod seed;
mod simplex;
mod systems;
mod tiling;
mod value;
mod warp;
mod worley;

pub use components::{
    NoiseGenerationCompleted, NoisePreviewConfig, NoisePreviewHandle, NoiseRegenerateRequested,
    NoiseRuntimeDiagnostics,
};
pub use config::{
    DomainTransform2, DomainTransform3, DomainTransform4, FractalConfig, GridSpace2, GridSpace3,
    NoiseRecipe2, NoiseRecipe4, PerlinConfig, RidgedConfig, SimplexConfig, TileConfig,
    ValueConfig, WarpConfig2, WarpConfig3, WorleyConfig, WorleyDistanceMetric, WorleyReturnType,
};
pub use fractal::{Billow, Fbm, Ridged, peak_amplitude_sum};
pub use grid::{
    Grid2, Grid3, GridRequest2, GridRequest3, GridSampleRequest, GridSampleResult, GridStats,
    MaskGrid2, generate_grid_sample, sample_grid2, sample_grid3,
};
pub use image::{
    GradientRamp, GradientStop, ImageNormalization, ImageOutputMode, NoiseImageSettings,
    grid_to_gradient_image, grid_to_grayscale_image, pack_scalar_layers_rgba,
};
pub use perlin::{Perlin, fade_curve};
pub use remap::{
    bias, binary_threshold, clamp_unit, contrast_pow, gain, remap_clamped, remap_range,
    signed_to_unit, smoothstep_threshold, unit_to_signed,
};
pub use sample::{NoiseRange, NoiseSource, NoiseSourceExt, RangeSemantics};
pub use seed::NoiseSeed;
pub use simplex::Simplex;
pub use tiling::{Tiled2, map_to_torus_4d};
pub use value::Value;
pub use warp::{DomainWarp2, DomainWarp3};
pub use worley::Worley;

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NoiseSystems {
    QueueJobs,
    PollJobs,
    UpdatePreview,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct NoisePlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
    pub preview: NoisePreviewConfig,
}

impl NoisePlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
            preview: NoisePreviewConfig::default(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }

    pub fn with_preview(mut self, preview: NoisePreviewConfig) -> Self {
        self.preview = preview;
        self
    }
}

impl Default for NoisePlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        if !app.world().contains_resource::<NoisePreviewConfig>() {
            app.insert_resource(self.preview.clone());
        }
        if !app.world().contains_resource::<NoisePreviewHandle>() {
            app.insert_resource(NoisePreviewHandle::default());
        }
        if !app.world().contains_resource::<NoiseRuntimeDiagnostics>() {
            app.insert_resource(NoiseRuntimeDiagnostics::default());
        }

        app.init_resource::<systems::NoiseRuntimeState>()
            .init_resource::<systems::PendingNoiseJob>()
            .init_resource::<systems::QueuedNoiseRequest>()
            .init_resource::<systems::PendingNoiseResult>()
            .add_message::<NoiseRegenerateRequested>()
            .add_message::<NoiseGenerationCompleted>()
            .register_type::<DomainTransform2>()
            .register_type::<DomainTransform3>()
            .register_type::<DomainTransform4>()
            .register_type::<FractalConfig>()
            .register_type::<GradientRamp>()
            .register_type::<GradientStop>()
            .register_type::<GridRequest2>()
            .register_type::<GridRequest3>()
            .register_type::<GridSpace2>()
            .register_type::<GridSpace3>()
            .register_type::<ImageNormalization>()
            .register_type::<ImageOutputMode>()
            .register_type::<NoiseImageSettings>()
            .register_type::<NoisePreviewConfig>()
            .register_type::<NoiseRecipe2>()
            .register_type::<NoiseRecipe4>()
            .register_type::<NoiseRuntimeDiagnostics>()
            .register_type::<NoiseSeed>()
            .register_type::<PerlinConfig>()
            .register_type::<RidgedConfig>()
            .register_type::<SimplexConfig>()
            .register_type::<TileConfig>()
            .register_type::<ValueConfig>()
            .register_type::<WarpConfig2>()
            .register_type::<WarpConfig3>()
            .register_type::<WorleyConfig>()
            .register_type::<WorleyDistanceMetric>()
            .register_type::<WorleyReturnType>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    NoiseSystems::QueueJobs,
                    NoiseSystems::PollJobs,
                    NoiseSystems::UpdatePreview,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                systems::queue_jobs
                    .in_set(NoiseSystems::QueueJobs)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::poll_jobs
                    .in_set(NoiseSystems::PollJobs)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::update_preview
                    .in_set(NoiseSystems::UpdatePreview)
                    .run_if(systems::runtime_is_active),
            );
    }
}

#[cfg(test)]
#[path = "noise_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
