use std::time::Duration;

use super::*;
use bevy::ecs::message::Messages;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

fn preview_request(async_generation: bool) -> GridSampleRequest {
    GridSampleRequest {
        recipe: NoiseRecipe2::Warp {
            base: Box::new(NoiseRecipe2::Fbm {
                source: Box::new(NoiseRecipe2::Perlin(PerlinConfig {
                    seed: NoiseSeed(301),
                })),
                config: FractalConfig {
                    octaves: 5,
                    base_frequency: 1.2,
                    ..default()
                },
            }),
            warp_x: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(302),
            })),
            warp_y: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(303),
            })),
            config: WarpConfig2 {
                amplitude: Vec2::splat(0.8),
                frequency: 1.7,
                ..default()
            },
        },
        grid: GridRequest2 {
            size: UVec2::new(48, 48),
            ..default()
        },
        image: NoiseImageSettings {
            gradient: GradientRamp::terrain(),
            ..default()
        },
        async_generation,
    }
}

fn alternate_preview_request(async_generation: bool) -> GridSampleRequest {
    GridSampleRequest {
        recipe: NoiseRecipe2::Ridged {
            source: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(404),
            })),
            config: RidgedConfig {
                fractal: FractalConfig {
                    octaves: 4,
                    base_frequency: 1.8,
                    gain: 0.6,
                    ..default()
                },
                ridge_offset: 1.15,
                weight_strength: 2.4,
            },
        },
        grid: GridRequest2 {
            size: UVec2::new(256, 256),
            ..default()
        },
        image: NoiseImageSettings {
            gradient: GradientRamp::heatmap(),
            ..default()
        },
        async_generation,
    }
}

fn test_app(request: GridSampleRequest) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Image>>();
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.add_plugins(
        NoisePlugin::new(Activate, Deactivate, Update).with_preview(NoisePreviewConfig { request }),
    );
    app.world_mut().run_schedule(Activate);
    app
}

fn preview_bytes(app: &App) -> Vec<u8> {
    let handle = app
        .world()
        .resource::<NoisePreviewHandle>()
        .0
        .clone()
        .expect("preview handle should exist");
    let image = app
        .world()
        .resource::<Assets<Image>>()
        .get(&handle)
        .expect("generated image should exist");
    image
        .data
        .clone()
        .expect("preview image should have pixel data")
}

#[test]
fn plugin_initializes_preview_resources() {
    let app = test_app(preview_request(false));

    assert!(app.world().contains_resource::<NoisePreviewConfig>());
    assert!(app.world().contains_resource::<NoisePreviewHandle>());
    assert!(app.world().contains_resource::<NoiseRuntimeDiagnostics>());
}

#[test]
fn sync_preview_generation_updates_resources_and_emits_message() {
    let mut app = test_app(preview_request(false));
    let mut cursor = app
        .world()
        .resource::<Messages<NoiseGenerationCompleted>>()
        .get_cursor();

    app.update();

    {
        let diagnostics = app.world().resource::<NoiseRuntimeDiagnostics>();
        assert_eq!(diagnostics.completed_jobs, 1);
        assert!(!diagnostics.task_running);
        assert_ne!(diagnostics.last_signature, 0);
        assert_eq!(diagnostics.grid_size, UVec2::new(48, 48));
        assert!(!diagnostics.async_generation);
        assert!(diagnostics.active_recipe.contains("Warp("));
    }

    let messages = app.world().resource::<Messages<NoiseGenerationCompleted>>();
    let completions: Vec<_> = cursor.read(messages).cloned().collect();
    assert_eq!(completions.len(), 1);

    let bytes = preview_bytes(&app);
    assert_eq!(bytes.len(), 48 * 48 * 4);
}

#[test]
fn async_and_sync_generation_match_for_the_same_request() {
    let sync_request = preview_request(false);
    let async_request = preview_request(true);

    let mut sync_app = test_app(sync_request);
    sync_app.update();
    let sync_diagnostics = sync_app
        .world()
        .resource::<NoiseRuntimeDiagnostics>()
        .clone();
    let sync_bytes = preview_bytes(&sync_app);

    let mut async_app = test_app(async_request);
    for _ in 0..256 {
        async_app.update();
        if async_app
            .world()
            .resource::<NoiseRuntimeDiagnostics>()
            .completed_jobs
            >= 1
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let async_diagnostics = async_app
        .world()
        .resource::<NoiseRuntimeDiagnostics>()
        .clone();
    assert_eq!(async_diagnostics.completed_jobs, 1);
    assert!(!async_diagnostics.task_running);
    assert_eq!(
        async_diagnostics.last_signature,
        sync_diagnostics.last_signature
    );
    assert!((async_diagnostics.last_min - sync_diagnostics.last_min).abs() <= 1.0e-6);
    assert!((async_diagnostics.last_max - sync_diagnostics.last_max).abs() <= 1.0e-6);
    assert_eq!(preview_bytes(&async_app), sync_bytes);
}

#[test]
fn async_runtime_requeues_latest_request_when_config_changes_mid_job() {
    let mut app = test_app(preview_request(true));

    app.update();

    {
        let diagnostics = app.world().resource::<NoiseRuntimeDiagnostics>();
        assert!(diagnostics.task_running);
        assert!(!diagnostics.pending_request);
    }

    {
        let mut preview = app.world_mut().resource_mut::<NoisePreviewConfig>();
        preview.request = alternate_preview_request(true);
    }

    app.update();
    assert!(
        app.world()
            .resource::<NoiseRuntimeDiagnostics>()
            .pending_request
    );

    for _ in 0..512 {
        app.update();
        if app
            .world()
            .resource::<NoiseRuntimeDiagnostics>()
            .completed_jobs
            >= 2
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let diagnostics = app.world().resource::<NoiseRuntimeDiagnostics>();
    assert_eq!(diagnostics.completed_jobs, 2);
    assert!(!diagnostics.task_running);
    assert!(!diagnostics.pending_request);
    assert_eq!(diagnostics.grid_size, UVec2::new(256, 256));
    assert!(diagnostics.active_recipe.contains("Ridged("));
}
