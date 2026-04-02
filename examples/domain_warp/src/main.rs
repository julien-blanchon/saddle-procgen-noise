use saddle_procgen_noise_example_common as support;

use bevy::prelude::*;
use saddle_procgen_noise::{
    DomainWarp2, Fbm, FractalConfig, GradientRamp, GridRequest2, NoiseImageSettings, NoiseSeed,
    NoiseSource, Perlin, Simplex, WarpConfig2, sample_grid2,
};

fn main() {
    let mut app = App::new();
    support::apply_window_defaults(
        &mut app,
        "noise domain warp example",
        (1500, 860),
        Color::srgb(0.04, 0.045, 0.06),
    );
    app.add_systems(Startup, setup).run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((Name::new("Domain Warp Camera"), Camera2d));

    let base = Fbm::new(
        Perlin::new(NoiseSeed(12)),
        FractalConfig {
            octaves: 5,
            base_frequency: 1.1,
            ..default()
        },
    );
    let warped = DomainWarp2::new(
        base,
        Simplex::new(NoiseSeed(24)),
        Simplex::new(NoiseSeed(25)),
        WarpConfig2 {
            amplitude: Vec2::splat(0.85),
            frequency: 1.9,
            ..default()
        },
    );

    let request = GridRequest2 {
        size: UVec2::new(320, 320),
        ..default()
    };
    let base_grid = sample_grid2(&base, &request);
    let warped_grid = sample_grid2(&warped, &request);
    let base_range = <Fbm<Perlin> as NoiseSource<Vec2>>::native_range(&base);
    let warped_range =
        <DomainWarp2<Fbm<Perlin>, Simplex, Simplex> as NoiseSource<Vec2>>::native_range(&warped);

    let settings = NoiseImageSettings {
        gradient: GradientRamp::heatmap(),
        ..default()
    };

    commands.spawn((
        Name::new("Warp Base"),
        Sprite::from_image(images.add(saddle_procgen_noise::grid_to_gradient_image(
            &base_grid,
            &settings,
            Some(base_range),
        ))),
        Transform::from_xyz(-280.0, 0.0, 0.0),
    ));
    commands.spawn((
        Name::new("Warp Result"),
        Sprite::from_image(images.add(saddle_procgen_noise::grid_to_gradient_image(
            &warped_grid,
            &settings,
            Some(warped_range),
        ))),
        Transform::from_xyz(280.0, 0.0, 0.0),
    ));

    commands.spawn((
        Name::new("Warp Label Base"),
        Text2d::new("Base FBM"),
        TextFont::from_font_size(22.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(-280.0, -210.0, 1.0),
    ));
    commands.spawn((
        Name::new("Warp Label Result"),
        Text2d::new("Warped"),
        TextFont::from_font_size(22.0),
        TextColor(Color::WHITE),
        Transform::from_xyz(280.0, -210.0, 1.0),
    ));
}
