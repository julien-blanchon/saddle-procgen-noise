use bevy::prelude::*;

use crate::grid::GridSampleRequest;

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct NoisePreviewConfig {
    pub request: GridSampleRequest,
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct NoisePreviewHandle(pub Option<Handle<Image>>);

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct NoiseRuntimeDiagnostics {
    pub active: bool,
    pub queued_jobs: u64,
    pub completed_jobs: u64,
    pub task_running: bool,
    pub pending_request: bool,
    pub grid_size: UVec2,
    pub async_generation: bool,
    pub active_recipe: String,
    pub last_signature: u64,
    pub last_duration_ms: f32,
    pub last_min: f32,
    pub last_max: f32,
    pub last_mean: f32,
    pub last_variance: f32,
}

#[derive(Debug, Clone, Message, Reflect, Default)]
pub struct NoiseRegenerateRequested {
    pub request_override: Option<GridSampleRequest>,
}

#[derive(Debug, Clone, Message, Reflect)]
pub struct NoiseGenerationCompleted {
    pub signature: u64,
    pub duration_ms: f32,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub variance: f32,
}
