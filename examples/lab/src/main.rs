#[cfg(feature = "e2e")]
mod e2e;

#[cfg(feature = "dev")]
use bevy::remote::{http::RemoteHttpPlugin, RemotePlugin};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_procgen_noise::{
    generate_grid_sample, GradientRamp, GridRequest2, GridSpace2, NoiseGenerationCompleted,
    NoiseImageSettings, NoisePlugin, NoisePreviewConfig, NoisePreviewHandle, NoiseRecipe2,
    NoiseRecipe4, NoiseRegenerateRequested, NoiseRuntimeDiagnostics, NoiseSeed, NoiseSource,
    NoiseSystems, PerlinConfig, RidgedConfig, SimplexConfig, TileConfig, ValueConfig, WorleyConfig,
    WorleyReturnType,
};

#[cfg(feature = "dev")]
const DEFAULT_BRP_PORT: u16 = 15_702;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub enum LabView {
    #[default]
    AsyncPreview,
    Compare,
    Seamless,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub enum AsyncPreset {
    #[default]
    Perlin,
    Value,
    Simplex,
    Worley,
    Fbm,
    Ridged,
    Warp,
}

#[derive(Debug, Clone, Resource, Reflect)]
#[reflect(Resource)]
struct LabControl {
    active_view: LabView,
    active_preset: AsyncPreset,
    seed: u32,
    pending_view: Option<LabView>,
    pending_preset: Option<AsyncPreset>,
    regenerate_requested: bool,
    last_signature: u64,
    recent_signatures: Vec<u64>,
}

impl Default for LabControl {
    fn default() -> Self {
        Self {
            active_view: LabView::AsyncPreview,
            active_preset: AsyncPreset::Perlin,
            seed: 13,
            pending_view: None,
            pending_preset: None,
            regenerate_requested: false,
            last_signature: 0,
            recent_signatures: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct LabDiagnostics {
    pub active_view: LabView,
    pub active_preset: AsyncPreset,
    pub seed: u32,
    pub preview_image_ready: bool,
    pub preview_size: UVec2,
    pub async_signature: u64,
    pub active_recipe: String,
    pub compare_panel_count: usize,
    pub compare_unique_signatures: usize,
    pub compare_signatures: Vec<u64>,
    pub completed_jobs: u64,
    pub queued_jobs: u64,
    pub task_running: bool,
    pub pending_request: bool,
    pub last_duration_ms: f32,
    pub recent_signatures: Vec<u64>,
    pub seamless_edge_delta: f32,
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct BeforeRegenerationSignature(pub u64);

#[derive(Resource)]
struct CompareArtifacts {
    signatures: Vec<u64>,
    seamless_edge_delta: f32,
}

#[derive(Component)]
struct AsyncRoot;

#[derive(Component)]
struct CompareRoot;

#[derive(Component)]
struct SeamlessRoot;

#[derive(Component)]
struct AsyncPreviewSprite;

#[derive(Component)]
struct OverlayText;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.055)));
    app.insert_resource(NoisePreviewConfig {
        request: async_request(AsyncPreset::Perlin, 13),
    });
    app.init_resource::<LabControl>();
    app.init_resource::<LabDiagnostics>();
    app.init_resource::<BeforeRegenerationSignature>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "noise crate-local lab".into(),
            resolution: (1520, 980).into(),
            ..default()
        }),
        ..default()
    }));
    #[cfg(feature = "dev")]
    app.add_plugins((
        RemotePlugin::default(),
        BrpExtrasPlugin::with_http_plugin(RemoteHttpPlugin::default().with_port(lab_brp_port())),
    ));
    app.add_plugins(NoisePlugin::default());
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::NoiseLabE2EPlugin);
    app.register_type::<LabView>();
    app.register_type::<AsyncPreset>();
    app.register_type::<LabControl>();
    app.register_type::<LabDiagnostics>();
    app.register_type::<BeforeRegenerationSignature>();
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            handle_keyboard_input,
            apply_lab_requests.before(NoiseSystems::QueueJobs),
            sync_async_preview_sprite.after(NoiseSystems::UpdatePreview),
            sync_diagnostics.after(NoiseSystems::UpdatePreview),
            sync_view_visibility,
            update_overlay.after(NoiseSystems::UpdatePreview),
        ),
    );
    app.run();
}

#[cfg(feature = "dev")]
fn lab_brp_port() -> u16 {
    std::env::var("BRP_EXTRAS_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_BRP_PORT)
}


fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Name::new("Noise Lab Camera"), Camera2d));
    let async_root = commands
        .spawn((
            Name::new("Noise Lab Async Root"),
            AsyncRoot,
            Transform::default(),
            Visibility::Visible,
        ))
        .id();
    let compare_root = commands
        .spawn((
            Name::new("Noise Lab Compare Root"),
            CompareRoot,
            Transform::default(),
            Visibility::Hidden,
        ))
        .id();
    let seamless_root = commands
        .spawn((
            Name::new("Noise Lab Seamless Root"),
            SeamlessRoot,
            Transform::default(),
            Visibility::Hidden,
        ))
        .id();

    let placeholder = images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[18, 24, 28, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    ));

    commands.entity(async_root).with_children(|parent| {
        parent.spawn((
            Name::new("Async Preview Sprite"),
            AsyncPreviewSprite,
            Sprite::from_image(placeholder),
            Transform::from_xyz(0.0, 20.0, 0.0),
        ));
    });

    commands.spawn((
        Name::new("Noise Lab Overlay"),
        OverlayText,
        Text::new("noise lab"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));

    let compare_artifacts =
        spawn_compare_panels(&mut commands, &mut images, compare_root, seamless_root);
    commands.insert_resource(compare_artifacts);
}

fn spawn_compare_panels(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    compare_root: Entity,
    seamless_root: Entity,
) -> CompareArtifacts {
    let presets = [
        ("Perlin", compare_recipe(AsyncPreset::Perlin)),
        ("Value", compare_recipe(AsyncPreset::Value)),
        ("Simplex", compare_recipe(AsyncPreset::Simplex)),
        ("Worley", compare_recipe(AsyncPreset::Worley)),
        ("FBM", compare_recipe(AsyncPreset::Fbm)),
        ("Ridged", compare_recipe(AsyncPreset::Ridged)),
        ("Warp", compare_recipe(AsyncPreset::Warp)),
    ];

    let mut signatures = Vec::with_capacity(presets.len());
    let panel_size = 240.0;
    commands.entity(compare_root).with_children(|parent| {
        for (index, (label, recipe)) in presets.into_iter().enumerate() {
            let row = index / 3;
            let col = index % 3;
            let x = -520.0 + col as f32 * 360.0;
            let y = 180.0 - row as f32 * 320.0;
            let result = generate_grid_sample(&saddle_procgen_noise::GridSampleRequest {
                recipe,
                grid: GridRequest2 {
                    size: UVec2::new(256, 256),
                    ..default()
                },
                image: NoiseImageSettings {
                    gradient: GradientRamp::heatmap(),
                    ..default()
                },
                async_generation: false,
            });
            signatures.push(result.signature);
            let handle = images.add(result.image);
            parent.spawn((
                Name::new(format!("Compare Panel {label}")),
                Sprite::from_image(handle),
                Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(panel_size / 256.0)),
            ));
            parent.spawn((
                Name::new(format!("Compare Label {label}")),
                Text2d::new(label),
                TextFont::from_font_size(20.0),
                TextColor(Color::WHITE),
                Transform::from_xyz(x, y - 148.0, 1.0),
            ));
        }
    });

    let seamless_recipe = saddle_procgen_noise::Tiled2::new(
        NoiseRecipe4::Fbm {
            source: Box::new(NoiseRecipe4::Simplex(SimplexConfig {
                seed: NoiseSeed(44),
            })),
            config: saddle_procgen_noise::FractalConfig {
                octaves: 4,
                base_frequency: 1.0,
                ..default()
            },
        },
        TileConfig {
            period: Vec2::splat(1.0),
        },
    );
    let seamless_grid = saddle_procgen_noise::sample_grid2(
        &seamless_recipe,
        &GridRequest2 {
            size: UVec2::new(256, 256),
            space: GridSpace2 {
                min: Vec2::ZERO,
                max: Vec2::ONE,
            },
        },
    );
    let seamless_delta = edge_delta(&seamless_grid);
    let seamless_image = images.add(saddle_procgen_noise::grid_to_gradient_image(
        &seamless_grid,
        &NoiseImageSettings {
            gradient: GradientRamp::terrain(),
            ..default()
        },
        Some(seamless_recipe.native_range()),
    ));
    let positions = [
        Vec3::new(-128.0, 128.0, 0.0),
        Vec3::new(128.0, 128.0, 0.0),
        Vec3::new(-128.0, -128.0, 0.0),
        Vec3::new(128.0, -128.0, 0.0),
    ];
    commands.entity(seamless_root).with_children(|parent| {
        for (index, position) in positions.into_iter().enumerate() {
            parent.spawn((
                Name::new(format!("Seamless Tile {}", index + 1)),
                Sprite::from_image(seamless_image.clone()),
                Transform::from_translation(position),
            ));
        }
        parent.spawn((
            Name::new("Seamless Label"),
            Text2d::new("One tile shown 2x2"),
            TextFont::from_font_size(22.0),
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, -310.0, 1.0),
        ));
    });

    CompareArtifacts {
        signatures,
        seamless_edge_delta: seamless_delta,
    }
}

fn handle_keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut control: ResMut<LabControl>) {
    if keys.just_pressed(KeyCode::Digit1) {
        control.pending_view = Some(LabView::AsyncPreview);
    } else if keys.just_pressed(KeyCode::Digit2) {
        control.pending_view = Some(LabView::Compare);
    } else if keys.just_pressed(KeyCode::Digit3) {
        control.pending_view = Some(LabView::Seamless);
    }

    if keys.just_pressed(KeyCode::KeyQ) {
        control.pending_preset = Some(AsyncPreset::Perlin);
    } else if keys.just_pressed(KeyCode::KeyW) {
        control.pending_preset = Some(AsyncPreset::Value);
    } else if keys.just_pressed(KeyCode::KeyE) {
        control.pending_preset = Some(AsyncPreset::Simplex);
    } else if keys.just_pressed(KeyCode::KeyR) {
        control.pending_preset = Some(AsyncPreset::Worley);
    } else if keys.just_pressed(KeyCode::KeyT) {
        control.pending_preset = Some(AsyncPreset::Fbm);
    } else if keys.just_pressed(KeyCode::KeyY) {
        control.pending_preset = Some(AsyncPreset::Ridged);
    } else if keys.just_pressed(KeyCode::KeyU) {
        control.pending_preset = Some(AsyncPreset::Warp);
    }

    if keys.just_pressed(KeyCode::Space) {
        control.regenerate_requested = true;
    }
}

fn apply_lab_requests(
    mut control: ResMut<LabControl>,
    mut writer: MessageWriter<NoiseRegenerateRequested>,
) {
    if let Some(view) = control.pending_view.take() {
        control.active_view = view;
    }

    if let Some(preset) = control.pending_preset.take() {
        control.active_preset = preset;
        writer.write(NoiseRegenerateRequested {
            request_override: Some(async_request(control.active_preset, control.seed)),
        });
        return;
    }

    if control.regenerate_requested {
        control.regenerate_requested = false;
        control.seed = control.seed.saturating_add(1);
        writer.write(NoiseRegenerateRequested {
            request_override: Some(async_request(control.active_preset, control.seed)),
        });
    }
}

fn sync_async_preview_sprite(
    preview: Res<NoisePreviewHandle>,
    mut sprite: Single<&mut Sprite, With<AsyncPreviewSprite>>,
) {
    let Some(handle) = preview.0.clone() else {
        return;
    };
    if sprite.image != handle {
        sprite.image = handle;
    }
}

fn sync_view_visibility(
    control: Res<LabControl>,
    mut async_root: Single<
        &mut Visibility,
        (With<AsyncRoot>, Without<CompareRoot>, Without<SeamlessRoot>),
    >,
    mut compare_root: Single<
        &mut Visibility,
        (With<CompareRoot>, Without<AsyncRoot>, Without<SeamlessRoot>),
    >,
    mut seamless_root: Single<
        &mut Visibility,
        (With<SeamlessRoot>, Without<AsyncRoot>, Without<CompareRoot>),
    >,
) {
    **async_root = if control.active_view == LabView::AsyncPreview {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    **compare_root = if control.active_view == LabView::Compare {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    **seamless_root = if control.active_view == LabView::Seamless {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn sync_diagnostics(
    mut control: ResMut<LabControl>,
    compare: Res<CompareArtifacts>,
    preview: Res<NoisePreviewHandle>,
    runtime: Res<NoiseRuntimeDiagnostics>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    if runtime.last_signature != 0 && runtime.last_signature != control.last_signature {
        control.last_signature = runtime.last_signature;
        control.recent_signatures.push(runtime.last_signature);
        if control.recent_signatures.len() > 12 {
            let drop = control.recent_signatures.len() - 12;
            control.recent_signatures.drain(0..drop);
        }
    }

    let mut unique = compare.signatures.clone();
    unique.sort_unstable();
    unique.dedup();

    *diagnostics = LabDiagnostics {
        active_view: control.active_view,
        active_preset: control.active_preset,
        seed: control.seed,
        preview_image_ready: preview.0.is_some() && runtime.last_signature != 0,
        preview_size: runtime.grid_size,
        async_signature: runtime.last_signature,
        active_recipe: runtime.active_recipe.clone(),
        compare_panel_count: compare.signatures.len(),
        compare_unique_signatures: unique.len(),
        compare_signatures: compare.signatures.clone(),
        completed_jobs: runtime.completed_jobs,
        queued_jobs: runtime.queued_jobs,
        task_running: runtime.task_running,
        pending_request: runtime.pending_request,
        last_duration_ms: runtime.last_duration_ms,
        recent_signatures: control.recent_signatures.clone(),
        seamless_edge_delta: compare.seamless_edge_delta,
    };
}

fn update_overlay(
    diagnostics: Res<LabDiagnostics>,
    mut completions: MessageReader<NoiseGenerationCompleted>,
    mut text: Single<&mut Text, With<OverlayText>>,
) {
    let completion_line = completions
        .read()
        .last()
        .map(|message| {
            format!(
                "last result: {:.2} ms [{:.3}, {:.3}]",
                message.duration_ms, message.min, message.max
            )
        })
        .unwrap_or_else(|| "last result: waiting".to_string());
    **text = format!(
        "View: {:?}\nPreset: {:?}\nSeed: {}\nPreview: {}x{}\nRecipe: {}\nAsync signature: {}\nCompare panels: {} ({} unique)\nQueued/completed: {}/{}\nRunning: {} | Pending: {}\nSeam delta: {:.6}\n{}\n1/2/3 switch views | Q..U change preset | Space regenerate",
        diagnostics.active_view,
        diagnostics.active_preset,
        diagnostics.seed,
        diagnostics.preview_size.x,
        diagnostics.preview_size.y,
        diagnostics.active_recipe,
        diagnostics.async_signature,
        diagnostics.compare_panel_count,
        diagnostics.compare_unique_signatures,
        diagnostics.queued_jobs,
        diagnostics.completed_jobs,
        diagnostics.task_running,
        diagnostics.pending_request,
        diagnostics.seamless_edge_delta,
        completion_line,
    )
    .into();
}

#[cfg(feature = "e2e")]
pub(crate) fn set_view(world: &mut World, view: LabView) {
    world.resource_mut::<LabControl>().pending_view = Some(view);
}

#[cfg(feature = "e2e")]
pub(crate) fn set_preset(world: &mut World, preset: AsyncPreset) {
    world.resource_mut::<LabControl>().pending_preset = Some(preset);
}

#[cfg(feature = "e2e")]
pub(crate) fn request_regeneration(world: &mut World) {
    world.resource_mut::<LabControl>().regenerate_requested = true;
}

fn async_request(preset: AsyncPreset, seed: u32) -> saddle_procgen_noise::GridSampleRequest {
    let recipe = match preset {
        AsyncPreset::Perlin => NoiseRecipe2::Perlin(PerlinConfig {
            seed: NoiseSeed(seed),
        }),
        AsyncPreset::Value => NoiseRecipe2::Value(ValueConfig {
            seed: NoiseSeed(seed),
        }),
        AsyncPreset::Simplex => NoiseRecipe2::Simplex(SimplexConfig {
            seed: NoiseSeed(seed),
        }),
        AsyncPreset::Worley => NoiseRecipe2::Worley(WorleyConfig {
            seed: NoiseSeed(seed),
            return_type: WorleyReturnType::F2MinusF1,
            ..default()
        }),
        AsyncPreset::Fbm => NoiseRecipe2::Fbm {
            source: Box::new(NoiseRecipe2::Perlin(PerlinConfig {
                seed: NoiseSeed(seed),
            })),
            config: saddle_procgen_noise::FractalConfig {
                octaves: 6,
                base_frequency: 1.25,
                ..default()
            },
        },
        AsyncPreset::Ridged => NoiseRecipe2::Ridged {
            source: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(seed),
            })),
            config: RidgedConfig {
                fractal: saddle_procgen_noise::FractalConfig {
                    octaves: 6,
                    base_frequency: 1.6,
                    ..default()
                },
                ..default()
            },
        },
        AsyncPreset::Warp => NoiseRecipe2::Warp {
            base: Box::new(NoiseRecipe2::Fbm {
                source: Box::new(NoiseRecipe2::Perlin(PerlinConfig {
                    seed: NoiseSeed(seed),
                })),
                config: saddle_procgen_noise::FractalConfig {
                    octaves: 5,
                    base_frequency: 1.1,
                    ..default()
                },
            }),
            warp_x: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(seed.saturating_add(1)),
            })),
            warp_y: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
                seed: NoiseSeed(seed.saturating_add(2)),
            })),
            config: saddle_procgen_noise::WarpConfig2 {
                amplitude: Vec2::splat(0.9),
                frequency: 1.8,
                ..default()
            },
        },
    };

    saddle_procgen_noise::GridSampleRequest {
        recipe,
        grid: GridRequest2 {
            size: UVec2::new(448, 448),
            space: GridSpace2 {
                min: Vec2::new(-2.0, -2.0),
                max: Vec2::new(2.0, 2.0),
            },
        },
        image: NoiseImageSettings {
            gradient: GradientRamp::terrain(),
            ..default()
        },
        async_generation: true,
    }
}

fn compare_recipe(preset: AsyncPreset) -> NoiseRecipe2 {
    async_request(preset, 7).recipe
}

fn edge_delta(grid: &saddle_procgen_noise::Grid2) -> f32 {
    let width = grid.size.x as usize;
    let height = grid.size.y as usize;
    let mut max_delta: f32 = 0.0;
    for y in 0..height {
        let left = grid.values[y * width];
        let right = grid.values[y * width + width - 1];
        max_delta = max_delta.max((left - right).abs());
    }
    for x in 0..width {
        let top = grid.values[x];
        let bottom = grid.values[(height - 1) * width + x];
        max_delta = max_delta.max((top - bottom).abs());
    }
    max_delta
}
