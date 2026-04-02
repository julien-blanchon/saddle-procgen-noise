use saddle_procgen_noise_example_common as support;

use bevy::prelude::*;
use saddle_procgen_noise::{
    FractalConfig, GradientRamp, GridRequest2, NoiseImageSettings, NoiseRecipe4, NoiseSeed,
    NoiseSource, SimplexConfig, TileConfig, Tiled2, sample_grid2,
};

fn main() {
    let mut app = App::new();
    support::apply_window_defaults(
        &mut app,
        "noise seamless example",
        (1200, 920),
        Color::srgb(0.035, 0.04, 0.055),
    );
    app.add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Name::new("Seamless Camera"), Camera2d));

    let tiled = Tiled2::new(
        NoiseRecipe4::Fbm {
            source: Box::new(NoiseRecipe4::Simplex(SimplexConfig {
                seed: NoiseSeed(33),
            })),
            config: FractalConfig {
                octaves: 4,
                base_frequency: 1.0,
                gain: 0.55,
                ..default()
            },
        },
        TileConfig {
            period: Vec2::splat(1.0),
        },
    );

    let grid = sample_grid2(
        &tiled,
        &GridRequest2 {
            size: UVec2::new(256, 256),
            space: saddle_procgen_noise::GridSpace2 {
                min: Vec2::ZERO,
                max: Vec2::ONE,
            },
        },
    );
    let handle = images.add(saddle_procgen_noise::grid_to_gradient_image(
        &grid,
        &NoiseImageSettings {
            gradient: GradientRamp::heatmap(),
            ..default()
        },
        Some(tiled.native_range()),
    ));

    let positions = [
        Vec3::new(-128.0, 128.0, 0.0),
        Vec3::new(128.0, 128.0, 0.0),
        Vec3::new(-128.0, -128.0, 0.0),
        Vec3::new(128.0, -128.0, 0.0),
    ];
    for (index, position) in positions.into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("Seamless Tile {}", index + 1)),
            Sprite::from_image(handle.clone()),
            Transform::from_translation(position),
        ));
    }

    commands.spawn((
        Name::new("Seamless Label"),
        Text2d::new("One generated tile, shown 2x2"),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -320.0, 1.0),
    ));
}
