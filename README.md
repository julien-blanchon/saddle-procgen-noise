# Saddle Procgen Noise

Deterministic noise generation utilities for reusable procedural workflows in Bevy.

`saddle-procgen-noise` is the mathematical foundation for terrain, cave, mask, texture, density-field, and runtime preview workflows. It is not a terrain system, biome system, voxel engine, or world generator.

## Quick Start

### Builder API (recommended)

```rust
use bevy::prelude::*;
use saddle_procgen_noise::{NoiseBuilder, GridRequest2, sample_grid2};

let recipe = NoiseBuilder::perlin()
    .seed(7)
    .fbm()
    .octaves(6)
    .frequency(1.2)
    .build();

let grid = sample_grid2(
    &recipe,
    &GridRequest2 {
        size: UVec2::new(128, 128),
        ..default()
    },
);

println!("min={} max={}", grid.stats.min, grid.stats.max);
```

### Pure Rust sampling (struct-level)

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

### Runtime Bevy integration

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

### Heightmap generation

```rust
use bevy::prelude::*;
use saddle_procgen_noise::{NoiseBuilder, GridRequest2, GridSpace2, generate_heightmap};

let recipe = NoiseBuilder::perlin().seed(42).fbm().octaves(6).build();
let grid = GridRequest2 {
    size: UVec2::new(128, 128),
    space: GridSpace2 { min: Vec2::splat(-3.0), max: Vec2::splat(3.0) },
};
let heights: Vec<f32> = generate_heightmap(&recipe, &grid); // normalized [0,1]
```

## Public API

| API | Purpose |
| --- | --- |
| `NoisePlugin` | Thin Bevy runtime layer for async preview-grid generation and image updates |
| `NoiseSystems` | Public runtime ordering hooks: `QueueJobs`, `PollJobs`, `UpdatePreview`, `Pipeline` |
| `NoiseSeed` | Explicit deterministic seed newtype |
| `NoiseSource<I>` | Core pure-Rust sampling trait for `Vec2`, `Vec3`, and `Vec4` domains |
| `NoiseBuilder` | Fluent builder API for constructing `NoiseRecipe2` pipelines |
| `Perlin`, `Simplex`, `Value`, `Worley` | Primitive samplers |
| `Fbm`, `Billow`, `Ridged` | Fractal combiners |
| `DomainWarp2`, `DomainWarp3` | Domain-warp wrappers |
| `Tiled2` | 2D seamless wrapper backed by 4D noise sampling |
| `PerlinConfig`, `SimplexConfig`, `ValueConfig`, `WorleyConfig` | Primitive sampler configuration |
| `FractalConfig`, `RidgedConfig`, `WarpConfig2`, `WarpConfig3`, `TileConfig` | Composition and tiling configuration |
| `NoiseRecipe2`, `NoiseRecipe4` | Recursive config-driven sampler recipes — serializable via Serde (RON/JSON) |
| `NoiseAsset` | Bevy Asset type for loading noise recipes from `.noise.ron` / `.noise.json` files |
| `NoiseImageGenerator` / `NoiseImageOutput` | Entity-driven noise-to-image pipeline components |
| `generate_heightmap` | Convenience function for terrain heightmap generation (normalized `[0,1]`) |
| `stamp_noise`, `modulate_grid`, `noise_mask` | Noise mask/brush utilities for runtime buffer manipulation |
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
- Value noise in 2D, 3D, and 4D
- Worley / cellular noise in 2D and 3D with `F1`, `F2`, `F2 - F1`, and Euclidean/Manhattan/Chebyshev metrics
- Fractal brownian motion, billow, and ridged-multifractal style variants
- Domain transforms, gain/bias/threshold/remap helpers, and conservative normalization helpers
- Multi-level domain warping (Quilez-style nested `f(p + fbm(p + fbm(p)))`)
- Tileable 2D sampling through 4D torus mapping
- Sync grid sampling for 2D images and 3D density batches
- Async preview-grid generation on Bevy task pools
- Grayscale, gradient, and RGBA channel-packed `Image` helpers
- Fluent `NoiseBuilder` API for composing noise recipes without constructing enums manually
- Serde serialization for all recipe types — load noise configs from `.noise.ron` or `.noise.json` asset files
- Entity-driven `NoiseImageGenerator` → `NoiseImageOutput` pipeline for reactive noise-to-image workflows
- `generate_heightmap()` for normalized terrain height extraction
- Noise mask utilities: `stamp_noise` (brush stamping), `modulate_grid` (erosion), `noise_mask` (binary thresholding)

## Examples

| Example | Description | Run |
| --- | --- | --- |
| `basic` | Pure sampling, deterministic spot checks, and timing for representative grids | `cargo run -p saddle-procgen-noise-example-basic` |
| `heightmap` | Side-by-side grayscale and terrain-gradient heightmap preview | `cargo run -p saddle-procgen-noise-example-heightmap` |
| `domain_warp` | Base field versus warped field comparison | `cargo run -p saddle-procgen-noise-example-domain-warp` |
| `seamless` | 2x2 tiled preview proving seamless wrapping | `cargo run -p saddle-procgen-noise-example-seamless` |
| `async_chunks` | Thin runtime-plugin example with async regeneration | `cargo run -p saddle-procgen-noise-example-async-chunks` |
| `gallery` | Visual catalog of all 10 noise types side-by-side with heatmap gradient | `cargo run -p saddle-procgen-noise-example-gallery` |
| `explorer` | Interactive noise parameter tweaking via keyboard (noise type, fractal mode, octaves, frequency, warp) | `cargo run -p saddle-procgen-noise-example-explorer` |
| `terrain` | 3D heightmap terrain mesh with vertex coloring (water/grass/rock/snow biomes) | `cargo run -p saddle-procgen-noise-example-terrain` |
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
