use saddle_procgen_noise_example_common as support;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use saddle_procgen_saddle_procgen_noise::{
    GradientRamp, GridRequest2, NoiseGenerationCompleted, NoiseImageSettings, NoisePlugin,
    NoisePreviewConfig, NoisePreviewHandle, NoiseRecipe2, NoiseRegenerateRequested,
    NoiseRuntimeDiagnostics, NoiseSeed, NoiseSystems, PerlinConfig,
};

#[derive(Component)]
struct PreviewSprite;

#[derive(Component)]
struct OverlayText;

#[derive(Resource)]
struct ExampleState {
    seed: u32,
}

fn main() {
    let mut app = App::new();
    support::apply_window_defaults(
        &mut app,
        "noise async preview example",
        (1180, 860),
        Color::srgb(0.035, 0.04, 0.055),
    );
    app.insert_resource(ExampleState { seed: 3 })
        .insert_resource(NoisePreviewConfig {
            request: preview_request(3),
        })
        .add_plugins(NoisePlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_input.before(NoiseSystems::QueueJobs),
                sync_preview_sprite.after(NoiseSystems::UpdatePreview),
                update_overlay.after(NoiseSystems::UpdatePreview),
            ),
        );
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Name::new("Async Preview Camera"), Camera2d));
    let placeholder = images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[20, 24, 28, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));
    commands.spawn((
        Name::new("Async Preview Sprite"),
        PreviewSprite,
        Sprite::from_image(placeholder),
    ));
    commands.spawn((
        Name::new("Async Preview Overlay"),
        OverlayText,
        Text::new("Space: regenerate"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ExampleState>,
    mut writer: MessageWriter<NoiseRegenerateRequested>,
) {
    if keys.just_pressed(KeyCode::Space) {
        state.seed = state.seed.saturating_add(1);
        writer.write(NoiseRegenerateRequested {
            request_override: Some(preview_request(state.seed)),
        });
    }
}

fn sync_preview_sprite(
    preview: Res<NoisePreviewHandle>,
    mut sprite: Single<&mut Sprite, With<PreviewSprite>>,
) {
    let Some(handle) = preview.0.clone() else {
        return;
    };
    if sprite.image != handle {
        sprite.image = handle;
    }
}

fn update_overlay(
    diagnostics: Res<NoiseRuntimeDiagnostics>,
    mut completions: MessageReader<NoiseGenerationCompleted>,
    mut text: Single<&mut Text, With<OverlayText>>,
) {
    let completion = completions.read().last().cloned();
    **text = if let Some(completion) = completion {
        format!(
            "Space: regenerate\nsignature: {}\nlast: {:.2} ms\nrange: [{:.3}, {:.3}]",
            completion.signature, completion.duration_ms, completion.min, completion.max
        )
        .into()
    } else {
        format!(
            "Space: regenerate\nqueued: {}\ncompleted: {}\nrunning: {}",
            diagnostics.queued_jobs, diagnostics.completed_jobs, diagnostics.task_running
        )
        .into()
    };
}

fn preview_request(seed: u32) -> saddle_procgen_noise::GridSampleRequest {
    saddle_procgen_noise::GridSampleRequest {
        recipe: NoiseRecipe2::Fbm {
            source: Box::new(NoiseRecipe2::Perlin(PerlinConfig {
                seed: NoiseSeed(seed),
            })),
            config: saddle_procgen_noise::FractalConfig {
                octaves: 6,
                base_frequency: 1.1,
                gain: 0.52,
                ..default()
            },
        },
        grid: GridRequest2 {
            size: UVec2::new(384, 384),
            ..default()
        },
        image: NoiseImageSettings {
            gradient: GradientRamp::terrain(),
            ..default()
        },
        async_generation: true,
    }
}
