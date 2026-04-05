use bevy::prelude::*;
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use saddle_procgen_noise::NoiseSystems;

use crate::{
    AsyncPreset, AsyncPreviewSprite, BeforeRegenerationSignature, LabDiagnostics, LabView,
    request_regeneration, set_preset, set_view,
};

pub struct NoiseLabE2EPlugin;

impl Plugin for NoiseLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(saddle_bevy_e2e::E2EPlugin);
        app.configure_sets(
            Update,
            saddle_bevy_e2e::E2ESet.before(NoiseSystems::QueueJobs),
        );
        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = parse_e2e_args(&args);
        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                }
                saddle_bevy_e2e::init_scenario(app, scenario);
            } else {
                error!(
                    "[noise_lab:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn parse_e2e_args(args: &[String]) -> (Option<String>, bool) {
    let mut scenario_name = None;
    let mut handoff = false;
    for arg in args.iter().skip(1) {
        if arg == "--handoff" {
            handoff = true;
        } else if !arg.starts_with('-') && scenario_name.is_none() {
            scenario_name = Some(arg.clone());
        }
    }
    if !handoff {
        handoff = std::env::var("E2E_HANDOFF").is_ok_and(|value| value == "1" || value == "true");
    }
    (scenario_name, handoff)
}

fn set_view_action(view: LabView) -> Action {
    Action::Custom(Box::new(move |world| set_view(world, view)))
}

fn set_preset_action(preset: AsyncPreset) -> Action {
    Action::Custom(Box::new(move |world| set_preset(world, preset)))
}

fn regenerate_action() -> Action {
    Action::Custom(Box::new(request_regeneration))
}

fn remember_signature() -> Action {
    Action::Custom(Box::new(|world| {
        world.resource_mut::<BeforeRegenerationSignature>().0 =
            world.resource::<LabDiagnostics>().async_signature;
    }))
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "noise_smoke" => Some(noise_smoke()),
        "noise_presets_compare" => Some(noise_presets_compare()),
        "noise_async_regen" => Some(noise_async_regen()),
        _ => None,
    }
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec!["noise_smoke", "noise_presets_compare", "noise_async_regen"]
}

fn wait_for_view(view: LabView) -> Action {
    Action::WaitUntil {
        label: format!("wait for {view:?}"),
        condition: Box::new(move |world| world.resource::<LabDiagnostics>().active_view == view),
        max_frames: 180,
    }
}

fn wait_for_preview_ready() -> Action {
    Action::WaitUntil {
        label: "wait for preview ready".into(),
        condition: Box::new(|world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.preview_image_ready && diagnostics.async_signature != 0
        }),
        max_frames: 240,
    }
}

fn noise_smoke() -> Scenario {
    Scenario::builder("noise_smoke")
        .description(
            "Verify the async preview, comparison grid, and diagnostics resources all initialize.",
        )
        .then(Action::WaitFrames(30))
        .then(wait_for_preview_ready())
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "comparison grid exists",
            |diagnostics| diagnostics.compare_panel_count >= 6,
        ))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "preview image ready",
            |diagnostics| diagnostics.preview_image_ready && diagnostics.async_signature != 0,
        ))
        .then(assertions::entity_exists::<AsyncPreviewSprite>(
            "async preview sprite exists",
        ))
        .then(assertions::resource_satisfies::<
            saddle_procgen_noise::NoiseRuntimeDiagnostics,
        >("runtime recipe stack recorded", |diagnostics| {
            !diagnostics.active_recipe.is_empty() && diagnostics.grid_size.x > 0
        }))
        .then(inspect::log_resource::<LabDiagnostics>(
            "noise_smoke_lab_diagnostics",
        ))
        .then(inspect::log_resource::<
            saddle_procgen_noise::NoiseRuntimeDiagnostics,
        >("noise_smoke_runtime_diagnostics"))
        .then(Action::Screenshot("noise_smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("noise_smoke"))
        .build()
}

fn noise_presets_compare() -> Scenario {
    Scenario::builder("noise_presets_compare")
        .description("Show the side-by-side preset grid and verify it contains distinct outputs.")
        .then(set_view_action(LabView::Compare))
        .then(wait_for_view(LabView::Compare))
        .then(Action::WaitFrames(12))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "compare grid has seven unique panels",
            |diagnostics| {
                diagnostics.compare_panel_count == 7 && diagnostics.compare_unique_signatures == 7
            },
        ))
        .then(inspect::log_resource::<LabDiagnostics>(
            "noise_compare_lab_diagnostics",
        ))
        .then(Action::Screenshot("noise_presets_compare".into()))
        .then(Action::WaitFrames(1))
        .then(set_view_action(LabView::Seamless))
        .then(wait_for_view(LabView::Seamless))
        .then(Action::WaitFrames(10))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "seamless edges match",
            |diagnostics| diagnostics.seamless_edge_delta <= 0.001,
        ))
        .then(inspect::log_resource::<LabDiagnostics>(
            "noise_seamless_lab_diagnostics",
        ))
        .then(Action::Screenshot("noise_seamless".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("noise_presets_compare"))
        .build()
}

fn noise_async_regen() -> Scenario {
    Scenario::builder("noise_async_regen")
        .description("Regenerate the async preview with a different preset and assert the published signature changes.")
        .then(set_view_action(LabView::AsyncPreview))
        .then(wait_for_view(LabView::AsyncPreview))
        .then(wait_for_preview_ready())
        .then(remember_signature())
        .then(set_preset_action(AsyncPreset::Warp))
        .then(Action::WaitUntil {
            label: "wait for warp preset preview".into(),
            condition: Box::new(|world| {
                let before = world.resource::<BeforeRegenerationSignature>().0;
                let diagnostics = world.resource::<LabDiagnostics>();
                diagnostics.active_preset == AsyncPreset::Warp
                    && diagnostics.async_signature != 0
                    && diagnostics.async_signature != before
            }),
            max_frames: 240,
        })
        .then(assertions::resource_satisfies::<saddle_procgen_noise::NoiseRuntimeDiagnostics>(
            "runtime recipe switched to warp",
            |diagnostics| diagnostics.active_recipe.contains("Warp("),
        ))
        .then(remember_signature())
        .then(inspect::log_resource::<LabDiagnostics>(
            "noise_async_before_regen",
        ))
        .then(Action::Screenshot("async_before".into()))
        .then(Action::WaitFrames(1))
        .then(regenerate_action())
        .then(Action::WaitUntil {
            label: "wait for new async signature".into(),
            condition: Box::new(|world| {
                let before = world.resource::<BeforeRegenerationSignature>().0;
                let diagnostics = world.resource::<LabDiagnostics>();
                diagnostics.async_signature != 0
                    && diagnostics.async_signature != before
                    && diagnostics.completed_jobs >= 2
            }),
            max_frames: 240,
        })
        .then(assertions::custom("async signature changed", |world| {
            let before = world.resource::<BeforeRegenerationSignature>().0;
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.async_signature != before
        }))
        .then(assertions::resource_satisfies::<LabDiagnostics>(
            "no queued request left behind",
            |diagnostics| !diagnostics.pending_request,
        ))
        .then(inspect::log_resource::<LabDiagnostics>(
            "noise_async_after_regen",
        ))
        .then(Action::Screenshot("async_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("noise_async_regen"))
        .build()
}
