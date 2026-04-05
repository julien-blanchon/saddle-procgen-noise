use saddle_procgen_noise_example_common as support;

use bevy::prelude::*;
use saddle_procgen_noise::{
    GradientRamp, GridRequest2, GridSpace2, NoiseBuilder, NoiseImageSettings, NoiseRecipe2,
    NoiseSeed, PerlinConfig, SimplexConfig, ValueConfig, WorleyConfig, WorleyReturnType,
    generate_grid_sample,
};

fn main() {
    let mut app = App::new();
    support::apply_window_defaults(
        &mut app,
        "noise gallery — all noise types",
        (1600, 960),
        Color::srgb(0.02, 0.025, 0.04),
    );
    app.add_systems(Startup, setup).run();
}

struct NoiseEntry {
    label: &'static str,
    description: &'static str,
    recipe: NoiseRecipe2,
}

fn noise_entries() -> Vec<NoiseEntry> {
    vec![
        NoiseEntry {
            label: "Perlin",
            description: "Classic gradient noise",
            recipe: NoiseRecipe2::Perlin(PerlinConfig { seed: NoiseSeed(7) }),
        },
        NoiseEntry {
            label: "Simplex",
            description: "Less axis bias",
            recipe: NoiseRecipe2::Simplex(SimplexConfig { seed: NoiseSeed(7) }),
        },
        NoiseEntry {
            label: "Value",
            description: "Hash-interpolated lattice",
            recipe: NoiseRecipe2::Value(ValueConfig { seed: NoiseSeed(7) }),
        },
        NoiseEntry {
            label: "Worley F1",
            description: "Cellular distance",
            recipe: NoiseRecipe2::Worley(WorleyConfig {
                seed: NoiseSeed(7),
                return_type: WorleyReturnType::F1,
                ..default()
            }),
        },
        NoiseEntry {
            label: "Worley F2-F1",
            description: "Cell boundaries",
            recipe: NoiseRecipe2::Worley(WorleyConfig {
                seed: NoiseSeed(7),
                return_type: WorleyReturnType::F2MinusF1,
                ..default()
            }),
        },
        NoiseEntry {
            label: "FBM Perlin",
            description: "6 octaves layered",
            recipe: NoiseBuilder::perlin()
                .seed(7)
                .fbm()
                .octaves(6)
                .frequency(1.2)
                .build(),
        },
        NoiseEntry {
            label: "Billow",
            description: "Cloud-like bumps",
            recipe: NoiseBuilder::simplex()
                .seed(7)
                .billow()
                .octaves(5)
                .frequency(1.3)
                .build(),
        },
        NoiseEntry {
            label: "Ridged",
            description: "Sharp mountain ridges",
            recipe: NoiseBuilder::simplex()
                .seed(7)
                .ridged()
                .octaves(6)
                .frequency(1.5)
                .build(),
        },
        NoiseEntry {
            label: "Domain Warp",
            description: "FBM with warped domain",
            recipe: NoiseBuilder::perlin()
                .seed(7)
                .fbm()
                .octaves(5)
                .frequency(1.1)
                .warp()
                .warp_amplitude(Vec2::splat(0.9))
                .warp_frequency(1.8)
                .build(),
        },
        NoiseEntry {
            label: "Multi-Warp",
            description: "Quilez nested warping",
            recipe: NoiseRecipe2::MultiWarp {
                base: Box::new(
                    NoiseBuilder::perlin()
                        .seed(7)
                        .fbm()
                        .octaves(5)
                        .frequency(1.0)
                        .build(),
                ),
                layers: vec![
                    NoiseBuilder::simplex()
                        .seed(42)
                        .fbm()
                        .octaves(4)
                        .frequency(1.2)
                        .build(),
                    NoiseBuilder::simplex()
                        .seed(99)
                        .fbm()
                        .octaves(4)
                        .frequency(1.2)
                        .build(),
                ],
                amplitude: 4.0,
            },
        },
    ]
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Name::new("Gallery Camera"), Camera2d));

    let entries = noise_entries();
    let cols = 5;
    let tile_size = 200.0;
    let spacing = 20.0;
    let total_w = cols as f32 * (tile_size + spacing) - spacing;
    let rows = (entries.len() + cols - 1) / cols;
    let total_h = rows as f32 * (tile_size + spacing + 30.0) - spacing;
    let start_x = -total_w / 2.0 + tile_size / 2.0;
    let start_y = total_h / 2.0 - tile_size / 2.0;

    let grid_request = GridRequest2 {
        size: UVec2::new(200, 200),
        space: GridSpace2 {
            min: Vec2::new(-2.5, -2.5),
            max: Vec2::new(2.5, 2.5),
        },
    };

    for (index, entry) in entries.into_iter().enumerate() {
        let col = index % cols;
        let row = index / cols;
        let x = start_x + col as f32 * (tile_size + spacing);
        let y = start_y - row as f32 * (tile_size + spacing + 30.0);

        let result = generate_grid_sample(&saddle_procgen_noise::GridSampleRequest {
            recipe: entry.recipe,
            grid: grid_request.clone(),
            image: NoiseImageSettings {
                gradient: GradientRamp::heatmap(),
                ..default()
            },
            async_generation: false,
        });

        let handle = images.add(result.image);
        commands.spawn((
            Name::new(format!("Gallery Tile {}", entry.label)),
            Sprite::from_image(handle),
            Transform::from_xyz(x, y, 0.0),
        ));

        // Label
        commands.spawn((
            Name::new(format!("Gallery Label {}", entry.label)),
            Text2d::new(entry.label),
            TextFont::from_font_size(16.0),
            TextColor(Color::WHITE),
            Transform::from_xyz(x, y - tile_size / 2.0 - 12.0, 1.0),
        ));

        // Description
        commands.spawn((
            Name::new(format!("Gallery Desc {}", entry.label)),
            Text2d::new(entry.description),
            TextFont::from_font_size(11.0),
            TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
            Transform::from_xyz(x, y - tile_size / 2.0 - 28.0, 1.0),
        ));
    }

    // Instructions
    commands.spawn((
        Name::new("Gallery Instructions"),
        Text::new("Noise Gallery — All noise types with heatmap gradient"),
        Node {
            position_type: PositionType::Absolute,
            top: px(8),
            left: px(12),
            ..default()
        },
    ));
}
