# Saddle Procgen Noise

Deterministic noise generation utilities for reusable procedural workflows in Bevy.

`saddle-procgen-noise` is the mathematical foundation for terrain, cave, mask, texture, density-field, and runtime preview workflows. It is not a terrain system, biome system, voxel engine, or world generator.

## Quick Start

Pure Rust sampling:

```rust
use bevy::prelude::*;
use saddle_procgen_noise::{Fbm, FractalConfig, GridRequest2, NoiseSeed, Perlin, sample_grid2};

let source = Fbm::new(
    Perlin::new(NoiseSeed(7)),
    FractalConfig {
        octaves: 5,
        base_frequency: 1.2,
        ..default()
    },
);

let grid = sample_grid2(
    &source,
    &GridRequest2 {
        size: UVec2::new(128, 128),
        ..default()
    },
);

println!("min={} max={}", grid.stats.min, grid.stats.max);
```

Runtime Bevy integration:

```rust,no_run
use bevy::prelude::*;
use saddle_procgen_noise::{NoisePlugin, NoisePreviewConfig};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.insert_resource(NoisePreviewConfig::default());
    app.add_plugins(NoisePlugin::default());
    app.run();
}
```

Use `NoisePlugin::new(activate, deactivate, update)` when you want the runtime preview layer to
follow your own schedules or app states.

## Public API

| API | Purpose |
| --- | --- |
| `NoisePlugin` | Thin Bevy runtime layer for async preview-grid generation and image updates |
| `NoiseSystems` | Public runtime ordering hooks: `QueueJobs`, `PollJobs`, `UpdatePreview` |
| `NoiseSeed` | Explicit deterministic seed newtype |
| `NoiseSource<I>` | Core pure-Rust sampling trait for `Vec2`, `Vec3`, and `Vec4` domains |
| `Perlin`, `Simplex`, `Worley` | Primitive samplers |
| `Fbm`, `Billow`, `Ridged` | Fractal combiners |
| `DomainWarp2`, `DomainWarp3` | Domain-warp wrappers |
| `Tiled2` | 2D seamless wrapper backed by 4D noise sampling |
| `PerlinConfig`, `SimplexConfig`, `WorleyConfig` | Primitive sampler configuration |
| `FractalConfig`, `RidgedConfig`, `WarpConfig2`, `WarpConfig3`, `TileConfig` | Composition and tiling configuration |
| `NoiseRecipe2`, `NoiseRecipe4` | Recursive config-driven sampler recipes for runtime or serialized workflows |
| `GridRequest2`, `GridRequest3`, `Grid2`, `Grid3` | Sync batch sampling primitives |
| `GridSampleRequest`, `GridSampleResult` | Recipe-driven preview-grid request/result types |
| `NoisePreviewConfig`, `NoiseRegenerateRequested`, `NoiseGenerationCompleted` | Runtime preview resources and messages |
| `NoiseImageSettings`, `GradientRamp` | Preview image output helpers |

## Range Semantics

Every sampler exposes `native_range()` returning a `NoiseRange { min, max, semantics }`.

- `RangeSemantics::Strict`: the published range is intended as a true bound.
- `RangeSemantics::Approximate`: the sampler is authored to live in that range, but the exact extrema depend on gradients or composition.
- `RangeSemantics::Conservative`: the bound is intentionally loose so normalization can stay deterministic without scanning data first.

Rules:

- `sample()` returns the native value without hidden remapping.
- `sample_normalized()` clamps the native value into `[0, 1]` using the published `NoiseRange`.
- Fractal combiners accumulate amplitude explicitly; they do not auto-normalize unless the caller asks for it.
- Grid image helpers can normalize from the sampler’s published range, the observed grid range, or an explicit caller-supplied range.

## What Ships

- Improved gradient Perlin noise in 2D, 3D, and 4D
- Simplex-style gradient noise in 2D, 3D, and 4D
- Worley / cellular noise in 2D and 3D with `F1`, `F2`, `F2 - F1`, and Euclidean/Manhattan/Chebyshev metrics
- Fractal brownian motion, billow, and ridged-multifractal style variants
- Domain transforms, gain/bias/threshold/remap helpers, and conservative normalization helpers
- Tileable 2D sampling through 4D torus mapping
- Sync grid sampling for 2D images and 3D density batches
- Async preview-grid generation on Bevy task pools
- Grayscale, gradient, and RGBA channel-packed `Image` helpers

## Examples

| Example | Description | Run |
| --- | --- | --- |
| `basic` | Pure sampling, deterministic spot checks, and timing for representative grids | `cargo run -p saddle-procgen-noise-example-basic` |
| `heightmap` | Side-by-side grayscale and terrain-gradient heightmap preview | `cargo run -p saddle-procgen-noise-example-heightmap` |
| `domain_warp` | Base field versus warped field comparison | `cargo run -p saddle-procgen-noise-example-domain-warp` |
| `seamless` | 2x2 tiled preview proving seamless wrapping | `cargo run -p saddle-procgen-noise-example-seamless` |
| `async_chunks` | Thin runtime-plugin example with async regeneration | `cargo run -p saddle-procgen-noise-example-async-chunks` |
| `saddle-procgen-noise-lab` | Rich crate-local BRP/E2E verification app | `cargo run -p saddle-procgen-noise-lab` |

## Performance Notes

- Cost scales linearly with grid size and octave count.
- Domain warping costs the base sample plus one sample per warp axis, per octave of the warp sources.
- Tileable 2D sampling is more expensive because it routes through 4D noise.
- Sync generation is suitable for direct tools and small previews; use the runtime async path for larger grids or live tuning.
- The crate avoids hidden allocations in hot sampling paths. Grid generation allocates once per output buffer.

Interactive examples can auto-exit for scripted verification with
`NOISE_EXAMPLE_EXIT_AFTER_SECONDS=<seconds>`.

See [docs/architecture.md](docs/architecture.md) for the math/runtime split and [docs/configuration.md](docs/configuration.md) for every public knob.
