use std::time::Instant;

use bevy::prelude::*;
use saddle_procgen_noise::{
    Fbm, FractalConfig, GridRequest2, GridRequest3, NoiseSeed, NoiseSource, Perlin, Simplex,
    TileConfig, Tiled2, Worley, WorleyConfig, WorleyReturnType, sample_grid2, sample_grid3,
};

fn main() {
    let perlin = Perlin::new(NoiseSeed(7));
    let simplex = Simplex::new(NoiseSeed(11));
    let worley = Worley::new(WorleyConfig {
        seed: NoiseSeed(19),
        return_type: WorleyReturnType::F2MinusF1,
        ..default()
    });

    let canonical = [
        Vec2::new(0.125, 0.25),
        Vec2::new(0.75, -0.5),
        Vec2::new(1.5, 2.25),
    ];

    println!("Canonical 2D samples");
    for point in canonical {
        println!(
            "point={point:?} perlin={:.5} simplex={:.5} worley={:.5}",
            perlin.sample(point),
            simplex.sample(point),
            worley.sample(point),
        );
    }

    let terrain = Fbm::new(
        perlin,
        FractalConfig {
            octaves: 6,
            base_frequency: 1.3,
            ..default()
        },
    );

    let started = Instant::now();
    let grid_256 = sample_grid2(
        &terrain,
        &GridRequest2 {
            size: UVec2::new(256, 256),
            ..default()
        },
    );
    println!(
        "256x256 fbm grid: {:.2} ms | min={:.3} max={:.3} mean={:.3} var={:.3}",
        started.elapsed().as_secs_f32() * 1000.0,
        grid_256.stats.min,
        grid_256.stats.max,
        grid_256.stats.mean,
        grid_256.stats.variance
    );

    let started = Instant::now();
    let grid_512 = sample_grid2(
        &terrain,
        &GridRequest2 {
            size: UVec2::new(512, 512),
            ..default()
        },
    );
    println!(
        "512x512 fbm grid: {:.2} ms | signature={}",
        started.elapsed().as_secs_f32() * 1000.0,
        grid_512.signature()
    );

    let started = Instant::now();
    let density = sample_grid3(
        &Simplex::new(NoiseSeed(33)),
        &GridRequest3 {
            size: UVec3::new(64, 64, 48),
            ..default()
        },
    );
    println!(
        "64x64x48 simplex density batch: {:.2} ms | min={:.3} max={:.3}",
        started.elapsed().as_secs_f32() * 1000.0,
        density.stats.min,
        density.stats.max
    );

    let tiled = Tiled2::new(
        Fbm::new(
            Simplex::new(NoiseSeed(41)),
            FractalConfig {
                octaves: 4,
                base_frequency: 1.0,
                ..default()
            },
        ),
        TileConfig {
            period: Vec2::splat(1.0),
        },
    );
    let left = tiled.sample(Vec2::new(0.0, 0.37));
    let right = tiled.sample(Vec2::new(1.0, 0.37));
    let top = tiled.sample(Vec2::new(0.62, 0.0));
    let bottom = tiled.sample(Vec2::new(0.62, 1.0));
    println!(
        "Seam check | horizontal delta={:.6} vertical delta={:.6}",
        (left - right).abs(),
        (top - bottom).abs()
    );
}
