use saddle_procgen_noise_example_common as support;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use saddle_procgen_noise::{
    GradientRamp, GridRequest2, GridSpace2, NoiseBuilder, NoiseGenerationCompleted,
    NoiseImageSettings, NoisePlugin, NoisePreviewConfig, NoisePreviewHandle, NoiseRecipe2,
    NoiseRegenerateRequested, NoiseRuntimeDiagnostics, NoiseSeed, NoiseSystems, SimplexConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoiseType {
    Perlin,
    Simplex,
    Value,
    Worley,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FractalType {
    None,
    Fbm,
    Billow,
    Ridged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GradientType {
    Grayscale,
    Terrain,
    Heatmap,
}

#[derive(Resource)]
struct ExplorerState {
    noise_type: NoiseType,
    fractal_type: FractalType,
    gradient_type: GradientType,
    seed: u32,
    octaves: u8,
    frequency: f32,
    lacunarity: f32,
    gain: f32,
    warp_enabled: bool,
    warp_amplitude: f32,
    dirty: bool,
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            noise_type: NoiseType::Perlin,
            fractal_type: FractalType::Fbm,
            gradient_type: GradientType::Terrain,
            seed: 7,
            octaves: 6,
            frequency: 1.2,
            lacunarity: 2.0,
            gain: 0.5,
            warp_enabled: false,
            warp_amplitude: 0.8,
            dirty: true,
        }
    }
}

impl ExplorerState {
    fn build_recipe(&self) -> NoiseRecipe2 {
        let base = match self.noise_type {
            NoiseType::Perlin => NoiseBuilder::perlin().seed(self.seed),
            NoiseType::Simplex => NoiseBuilder::simplex().seed(self.seed),
            NoiseType::Value => NoiseBuilder::value().seed(self.seed),
            NoiseType::Worley => NoiseBuilder::worley().seed(self.seed),
        };

        let recipe = match self.fractal_type {
            FractalType::None => base.build(),
            FractalType::Fbm => base
                .fbm()
                .octaves(self.octaves)
                .frequency(self.frequency)
                .lacunarity(self.lacunarity)
                .gain(self.gain)
                .build(),
            FractalType::Billow => base
                .billow()
                .octaves(self.octaves)
                .frequency(self.frequency)
                .lacunarity(self.lacunarity)
                .gain(self.gain)
                .build(),
            FractalType::Ridged => base
                .ridged()
                .octaves(self.octaves)
                .frequency(self.frequency)
                .lacunarity(self.lacunarity)
                .gain(self.gain)
                .build(),
        };

        if self.warp_enabled {
            NoiseRecipe2::Warp {
                base: Box::new(recipe),
                warp_x: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
                    seed: NoiseSeed(self.seed.wrapping_add(100)),
                })),
                warp_y: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
                    seed: NoiseSeed(self.seed.wrapping_add(200)),
                })),
                config: saddle_procgen_noise::WarpConfig2 {
                    amplitude: Vec2::splat(self.warp_amplitude),
                    frequency: 1.5,
                    ..default()
                },
            }
        } else {
            recipe
        }
    }

    fn gradient(&self) -> GradientRamp {
        match self.gradient_type {
            GradientType::Grayscale => GradientRamp::grayscale(),
            GradientType::Terrain => GradientRamp::terrain(),
            GradientType::Heatmap => GradientRamp::heatmap(),
        }
    }
}

#[derive(Component)]
struct PreviewSprite;

#[derive(Component)]
struct OverlayText;

fn main() {
    let mut app = App::new();
    support::apply_window_defaults(
        &mut app,
        "noise explorer — interactive parameter tweaking",
        (1300, 900),
        Color::srgb(0.02, 0.025, 0.04),
    );
    app.init_resource::<ExplorerState>()
        .insert_resource(NoisePreviewConfig {
            request: make_request(&ExplorerState::default()),
        })
        .add_plugins(NoisePlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_input,
                apply_changes.before(NoiseSystems::QueueJobs),
                sync_preview_sprite.after(NoiseSystems::UpdatePreview),
                update_overlay.after(NoiseSystems::UpdatePreview),
            ),
        );
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Name::new("Explorer Camera"), Camera2d));
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
        Name::new("Explorer Preview Sprite"),
        PreviewSprite,
        Sprite::from_image(placeholder),
        Transform::from_xyz(100.0, 0.0, 0.0),
    ));
    commands.spawn((
        Name::new("Explorer Overlay"),
        OverlayText,
        Text::new("loading..."),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn handle_input(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<ExplorerState>) {
    // Noise type
    if keys.just_pressed(KeyCode::Digit1) {
        state.noise_type = NoiseType::Perlin;
        state.dirty = true;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        state.noise_type = NoiseType::Simplex;
        state.dirty = true;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        state.noise_type = NoiseType::Value;
        state.dirty = true;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        state.noise_type = NoiseType::Worley;
        state.dirty = true;
    }

    // Fractal type
    if keys.just_pressed(KeyCode::KeyF) {
        state.fractal_type = match state.fractal_type {
            FractalType::None => FractalType::Fbm,
            FractalType::Fbm => FractalType::Billow,
            FractalType::Billow => FractalType::Ridged,
            FractalType::Ridged => FractalType::None,
        };
        state.dirty = true;
    }

    // Gradient
    if keys.just_pressed(KeyCode::KeyG) {
        state.gradient_type = match state.gradient_type {
            GradientType::Grayscale => GradientType::Terrain,
            GradientType::Terrain => GradientType::Heatmap,
            GradientType::Heatmap => GradientType::Grayscale,
        };
        state.dirty = true;
    }

    // Warp toggle
    if keys.just_pressed(KeyCode::KeyW) {
        state.warp_enabled = !state.warp_enabled;
        state.dirty = true;
    }

    // Seed
    if keys.just_pressed(KeyCode::Space) {
        state.seed = state.seed.wrapping_add(1);
        state.dirty = true;
    }

    // Octaves
    if keys.just_pressed(KeyCode::ArrowUp) {
        state.octaves = (state.octaves + 1).min(10);
        state.dirty = true;
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        state.octaves = state.octaves.saturating_sub(1).max(1);
        state.dirty = true;
    }

    // Frequency
    if keys.just_pressed(KeyCode::ArrowRight) {
        state.frequency = (state.frequency + 0.1).min(10.0);
        state.dirty = true;
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        state.frequency = (state.frequency - 0.1).max(0.1);
        state.dirty = true;
    }

    // Lacunarity
    if keys.just_pressed(KeyCode::KeyL) {
        state.lacunarity = if state.lacunarity >= 3.5 {
            1.5
        } else {
            state.lacunarity + 0.25
        };
        state.dirty = true;
    }

    // Gain
    if keys.just_pressed(KeyCode::KeyH) {
        state.gain = if state.gain >= 0.9 {
            0.2
        } else {
            state.gain + 0.1
        };
        state.dirty = true;
    }
}

fn apply_changes(
    mut state: ResMut<ExplorerState>,
    mut writer: MessageWriter<NoiseRegenerateRequested>,
) {
    if !state.dirty {
        return;
    }
    state.dirty = false;
    writer.write(NoiseRegenerateRequested {
        request_override: Some(make_request(&state)),
    });
}

fn make_request(state: &ExplorerState) -> saddle_procgen_noise::GridSampleRequest {
    saddle_procgen_noise::GridSampleRequest {
        recipe: state.build_recipe(),
        grid: GridRequest2 {
            size: UVec2::new(512, 512),
            space: GridSpace2 {
                min: Vec2::new(-3.0, -3.0),
                max: Vec2::new(3.0, 3.0),
            },
        },
        image: NoiseImageSettings {
            gradient: state.gradient(),
            ..default()
        },
        async_generation: true,
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
    state: Res<ExplorerState>,
    diagnostics: Res<NoiseRuntimeDiagnostics>,
    mut completions: MessageReader<NoiseGenerationCompleted>,
    mut text: Single<&mut Text, With<OverlayText>>,
) {
    let last = completions.read().last().cloned();
    let timing = last
        .map(|c| format!("{:.1}ms", c.duration_ms))
        .unwrap_or_else(|| "...".into());

    **text = format!(
        "NOISE EXPLORER\n\
         \n\
         Noise:    {:?}        [1-4]\n\
         Fractal:  {:?}      [F]\n\
         Gradient: {:?}     [G]\n\
         Warp:     {}          [W]\n\
         \n\
         Seed:     {}           [Space]\n\
         Octaves:  {}           [Up/Down]\n\
         Freq:     {:.1}        [Left/Right]\n\
         Lacun:    {:.2}        [L]\n\
         Gain:     {:.2}        [H]\n\
         \n\
         Gen time: {}\n\
         Recipe:   {}",
        state.noise_type,
        state.fractal_type,
        state.gradient_type,
        if state.warp_enabled { "ON" } else { "OFF" },
        state.seed,
        state.octaves,
        state.frequency,
        state.lacunarity,
        state.gain,
        timing,
        diagnostics.active_recipe,
    )
    .into();
}
