use saddle_procgen_noise_example_common as support;

use bevy::prelude::*;
use saddle_procgen_noise::{
    Fbm, FractalConfig, GradientRamp, GridRequest2, NoiseImageSettings, NoiseSeed, NoiseSource,
    Perlin, grid_to_grayscale_image, sample_grid2,
};

fn main() {
    let mut app = App::new();
    support::apply_window_defaults(
        &mut app,
        "noise heightmap example",
        (1400, 840),
        Color::srgb(0.05, 0.06, 0.08),
    );
    app.add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Name::new("Heightmap Camera"), Camera2d));

    let source = Fbm::new(
        Perlin::new(NoiseSeed(5)),
        FractalConfig {
            octaves: 6,
            base_frequency: 1.1,
            lacunarity: 2.1,
            gain: 0.52,
            ..default()
        },
    );
    let grid = sample_grid2(
        &source,
        &GridRequest2 {
            size: UVec2::new(320, 320),
            ..default()
        },
    );
    let range = <Fbm<Perlin> as NoiseSource<Vec2>>::native_range(&source);

    let gray = images.add(grid_to_grayscale_image(&grid, Some(range)));
    let terrain = images.add(saddle_procgen_noise::grid_to_gradient_image(
        &grid,
        &NoiseImageSettings {
            gradient: GradientRamp::terrain(),
            ..default()
        },
        Some(range),
    ));

    commands.spawn((
        Name::new("Heightmap Grayscale"),
        Sprite::from_image(gray),
        Transform::from_xyz(-260.0, 0.0, 0.0),
    ));
    commands.spawn((
        Name::new("Heightmap Terrain"),
        Sprite::from_image(terrain),
        Transform::from_xyz(260.0, 0.0, 0.0),
    ));

    commands.spawn((
        Name::new("Heightmap Label Left"),
        Text2d::new("Grayscale"),
        TextFont::from_font_size(22.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(-260.0, -210.0, 1.0),
    ));
    commands.spawn((
        Name::new("Heightmap Label Right"),
        Text2d::new("Gradient"),
        TextFont::from_font_size(22.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(260.0, -210.0, 1.0),
    ));
}
