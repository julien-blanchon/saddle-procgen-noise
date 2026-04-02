use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};

use crate::{
    components::{
        NoiseGenerationCompleted, NoisePreviewConfig, NoisePreviewHandle, NoiseRegenerateRequested,
        NoiseRuntimeDiagnostics,
    },
    grid::{GridSampleRequest, GridSampleResult, generate_grid_sample},
};

#[derive(Default, Resource)]
pub(crate) struct NoiseRuntimeState {
    pub active: bool,
}

#[derive(Default, Resource)]
pub(crate) struct PendingNoiseJob {
    pub task: Option<Task<GridSampleResult>>,
    pub request: Option<GridSampleRequest>,
}

#[derive(Default, Resource)]
pub(crate) struct PendingNoiseResult(pub Option<GridSampleResult>);

#[derive(Default, Resource)]
pub(crate) struct QueuedNoiseRequest(pub Option<GridSampleRequest>);

pub(crate) fn activate_runtime(
    mut state: ResMut<NoiseRuntimeState>,
    mut diagnostics: ResMut<NoiseRuntimeDiagnostics>,
) {
    state.active = true;
    diagnostics.active = true;
}

pub(crate) fn deactivate_runtime(
    mut state: ResMut<NoiseRuntimeState>,
    mut job: ResMut<PendingNoiseJob>,
    mut queued: ResMut<QueuedNoiseRequest>,
    mut diagnostics: ResMut<NoiseRuntimeDiagnostics>,
) {
    state.active = false;
    job.task = None;
    job.request = None;
    queued.0 = None;
    diagnostics.active = false;
    diagnostics.task_running = false;
    diagnostics.pending_request = false;
}

pub(crate) fn runtime_is_active(state: Res<NoiseRuntimeState>) -> bool {
    state.active
}

pub(crate) fn queue_jobs(
    mut preview: ResMut<NoisePreviewConfig>,
    mut requests: MessageReader<NoiseRegenerateRequested>,
    mut job: ResMut<PendingNoiseJob>,
    mut queued: ResMut<QueuedNoiseRequest>,
    mut pending: ResMut<PendingNoiseResult>,
    mut diagnostics: ResMut<NoiseRuntimeDiagnostics>,
) {
    let mut explicit_request = false;
    for request in requests.read() {
        explicit_request = true;
        if let Some(override_request) = &request.request_override {
            preview.request = override_request.clone();
        }
    }

    let desired_request = preview.request.clone();
    let desired_changed = preview.is_added() || preview.is_changed() || explicit_request;

    diagnostics.grid_size = desired_request.grid.size;
    diagnostics.async_generation = desired_request.async_generation;
    diagnostics.active_recipe = desired_request.recipe.debug_stack();

    if job.task.is_some() {
        if desired_changed {
            if job.request.as_ref() == Some(&desired_request) {
                queued.0 = None;
            } else {
                queued.0 = Some(desired_request);
            }
        }
        diagnostics.pending_request = queued.0.is_some();
        return;
    }

    let should_generate = diagnostics.completed_jobs == 0 || desired_changed || queued.0.is_some();
    if !should_generate {
        diagnostics.pending_request = false;
        return;
    }

    let request_to_generate = queued.0.take().unwrap_or(desired_request);
    diagnostics.pending_request = queued.0.is_some();
    diagnostics.queued_jobs = diagnostics.queued_jobs.saturating_add(1);

    if request_to_generate.async_generation {
        diagnostics.task_running = true;
        job.request = Some(request_to_generate.clone());
        job.task = Some(
            AsyncComputeTaskPool::get()
                .spawn(async move { generate_grid_sample(&request_to_generate) }),
        );
    } else {
        diagnostics.task_running = false;
        job.request = None;
        pending.0 = Some(generate_grid_sample(&request_to_generate));
    }
}

pub(crate) fn poll_jobs(
    mut job: ResMut<PendingNoiseJob>,
    mut pending: ResMut<PendingNoiseResult>,
    mut diagnostics: ResMut<NoiseRuntimeDiagnostics>,
) {
    let Some(task) = job.task.as_mut() else {
        return;
    };
    if let Some(result) = check_ready(task) {
        job.task = None;
        job.request = None;
        diagnostics.task_running = false;
        pending.0 = Some(result);
    }
}

pub(crate) fn update_preview(
    mut pending: ResMut<PendingNoiseResult>,
    mut images: ResMut<Assets<Image>>,
    mut handle: ResMut<NoisePreviewHandle>,
    mut diagnostics: ResMut<NoiseRuntimeDiagnostics>,
    mut writer: MessageWriter<NoiseGenerationCompleted>,
) {
    let Some(result) = pending.0.take() else {
        return;
    };

    if let Some(existing) = handle
        .0
        .as_ref()
        .and_then(|existing| images.get_mut(existing))
    {
        *existing = result.image.clone();
    } else {
        handle.0 = Some(images.add(result.image.clone()));
    }

    diagnostics.completed_jobs = diagnostics.completed_jobs.saturating_add(1);
    diagnostics.last_signature = result.signature;
    diagnostics.last_duration_ms = result.duration_ms;
    diagnostics.last_min = result.grid.stats.min;
    diagnostics.last_max = result.grid.stats.max;
    diagnostics.last_mean = result.grid.stats.mean;
    diagnostics.last_variance = result.grid.stats.variance;

    writer.write(NoiseGenerationCompleted {
        signature: result.signature,
        duration_ms: result.duration_ms,
        min: result.grid.stats.min,
        max: result.grid.stats.max,
        mean: result.grid.stats.mean,
        variance: result.grid.stats.variance,
    });
}
