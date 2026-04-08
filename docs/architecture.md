# Architecture

`saddle-procgen-noise` is split into six layers.

## 1. Primitive Functions

- `perlin.rs`: improved lattice gradient noise in 2D, 3D, and 4D
- `simplex.rs`: simplex-style gradient noise in 2D, 3D, and 4D
- `value.rs`: hash-interpolated lattice noise in 2D, 3D, and 4D
- `worley.rs`: cellular feature-point distance noise in 2D and 3D
- `hash.rs`: deterministic integer hashing for lattice points and feature-point jitter
- `seed.rs`: explicit seed type with `Serialize`/`Deserialize` support, no global RNG state

All primitive sampling is pure Rust and deterministic from `(seed, coordinates)`.

## 2. Composition

- `fractal.rs`: FBM, billow, and ridged accumulation
- `warp.rs`: domain-warp wrappers for 2D and 3D
- `tiling.rs`: 2D seamless mapping through a 4D torus transform
- `remap.rs`: clamp, remap, gain, bias, contrast, and threshold helpers
- `config.rs`: caller-owned configuration objects and recursive recipe enums (all `Serialize`/`Deserialize`)
- `builder.rs`: fluent `NoiseBuilder` API for constructing `NoiseRecipe2` pipelines without manual enum construction

The `NoiseRecipe2` enum now includes a `MultiWarp` variant for Quilez-style multi-level domain warping (`f(p + fbm(p + fbm(p)))`).

Composition stays outside ECS. Runtime systems only orchestrate jobs and image publication.

## 3. Batch Outputs

- `grid.rs`: sync 2D and 3D sampling, signatures, masks, and statistics
- `image.rs`: grayscale, gradient, and packed-channel `Image` creation

The grid layer is the bridge between pure sampling and runtime orchestration. It is still usable without a Bevy `App`.

## 4. Higher-Level Utilities

- `mask.rs`: noise mask/brush tools — `stamp_noise()` for brush-stamping noise onto buffers with quadratic falloff, `modulate_grid()` for erosion/weathering, `noise_mask()` for binary threshold masks
- `pipeline.rs`: entity-driven `NoiseImageGenerator` → `NoiseImageOutput` component pipeline. Watches `Changed<NoiseImageGenerator>` and produces `Image` handles automatically. Also provides `generate_heightmap()` for terrain workflows.

## 5. Asset Integration

- `asset.rs` (behind `asset` feature flag): `NoiseAsset` as a Bevy `Asset` + `TypePath`, with `NoiseRonAssetLoader` (`.noise.ron`) and `NoiseJsonAssetLoader` (`.noise.json`). Recipes can be authored as data files and loaded via Bevy's asset system.

## 6. Runtime Integration

- `components.rs`: preview resources and messages
- `systems.rs`: async queueing, task polling, and preview-image updates
- `lib.rs`: `NoisePlugin` and public `NoiseSystems` (now includes `Pipeline` set)

The runtime layer intentionally does not model the math as ECS. It only manages:

- config changes
- async job launch
- preserving the latest requested config while an async job is already in flight
- job completion
- publishing `Image` assets and diagnostics

## Hash / Lattice Strategy

- Integer lattice coordinates are hashed with a small owned 32-bit avalanche mixer.
- Primitive samplers never allocate or mutate shared state.
- Perlin and simplex gradients are selected from fixed direction tables using hashed corner IDs.
- Worley feature points are deterministic jittered points inside each cell, derived from hashed axis-specific salts.

## Tiling Strategy

Tileable 2D output is implemented by mapping `(x, y)` to 4D torus coordinates:

- `x` becomes `(cos(ax), sin(ax))`
- `y` becomes `(cos(ay), sin(ay))`

This guarantees matching opposite edges as long as the underlying source supports 4D sampling. Tradeoff: seamless mode costs a 4D sample, so it is slower than direct 2D noise.

## Perlin vs Simplex vs Worley

### Perlin

- Regular lattice interpolation
- Easy to reason about
- Good default for terrain-like continuous fields
- More visible axial structure than simplex under some transforms

### Simplex

- Simplicial cell structure instead of axis-aligned cubes
- Usually reduces visible axis bias
- Scales better than classic lattice interpolation in higher dimensions
- Slightly more complex implementation and tuning surface

### Worley

- Feature-point distance noise
- Useful for clustered masks, crackle, cell boundaries, and placement masks
- Returns distance-derived values such as `F1`, `F2`, or `F2 - F1`
- Conservative normalization is used because exact distance maxima depend on configuration and metric

## Async Job Flow

`NoisePlugin` runs three ordered phases:

1. `QueueJobs`
2. `PollJobs`
3. `UpdatePreview`

Flow:

1. A `NoisePreviewConfig` resource changes, or `NoiseRegenerateRequested` arrives.
2. The plugin either spawns a background task or computes immediately for sync mode.
3. If the config changes while a task is still running, the latest request is queued and replayed
   after the in-flight task completes instead of being dropped.
4. Completed results are published into `Assets<Image>` and reflected diagnostics resources.
5. Consumers such as examples or labs bind the published `Handle<Image>` to sprites or UI.

This keeps the public runtime surface generic while preserving a fully pure-Rust core.

## CPU vs GPU Design Decision

All noise algorithms in this crate are **CPU-only** by design. This is intentional, not a limitation to be worked around.

### Why CPU

- **Determinism** — CPU floating-point behavior is consistent across hardware. GPU float precision varies between vendors and driver versions, making reproducible world generation unreliable.
- **Testability** — pure Rust functions are trivially unit-testable with `proptest` and standard assertions. GPU compute requires a render context.
- **Composability** — recursive `NoiseRecipe2` enums allow arbitrary nesting (FBM of warped ridged simplex). GPU shader branching is limited and dynamic dispatch is expensive.
- **Serialization** — recipes serialize to RON/JSON for asset pipelines and tooling. Shader parameters require custom encoding.
- **No render context** — sampling works without a Bevy `App`, in CLI tools, tests, or offline bakers.

### When GPU Noise Is Better

For anything sampled **per-vertex or per-pixel every frame**, CPU noise is the wrong tool:

- Wind displacement (per-vertex in vertex shader)
- Grass blade variation (per-blade)
- Water surface animation (per-vertex)
- Dissolve / shader VFX (per-pixel in fragment shader)

These should use inline WGSL noise functions. The algorithms are the same (Perlin, Simplex, FBM) but execute on the GPU's massively parallel architecture.

### Hybrid Pattern

The recommended bridge: use this crate to **bake noise into textures** at startup or per-chunk, then sample those textures in GPU shaders via standard texture reads. The `pack_scalar_layers_rgba()` function packs up to 4 float grids into RGBA channels for exactly this workflow.
