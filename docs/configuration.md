# Configuration

## Primitive Samplers

### `PerlinConfig`

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `seed` | `NoiseSeed` | `0` | Deterministic seed for gradient hashing |

### `SimplexConfig`

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `seed` | `NoiseSeed` | `0` | Deterministic seed for simplex corner gradients |

### `WorleyConfig`

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `seed` | `NoiseSeed` | `0` | Deterministic feature-point seed |
| `jitter` | `f32` | `1.0` | `0` centers points in each cell, `1` uses the full cell interior |
| `distance` | `WorleyDistanceMetric` | `Euclidean` | Distance metric for sorting feature points |
| `return_type` | `WorleyReturnType` | `F1` | `F1`, `F2`, or `F2 - F1` |

## Fractals

### `FractalConfig`

| Field | Type | Default | Visual Effect | Cost |
| --- | --- | --- | --- | --- |
| `octaves` | `u8` | `5` | More detail layers | Linear in octave count |
| `base_frequency` | `f32` | `1.0` | Global detail scale | Cheap |
| `lacunarity` | `f32` | `2.0` | Frequency multiplier per octave | Cheap |
| `gain` | `f32` | `0.5` | Amplitude falloff per octave | Cheap |
| `amplitude` | `f32` | `1.0` | Overall output strength | Cheap |

### `RidgedConfig`

| Field | Type | Default | Visual Effect |
| --- | --- | --- | --- |
| `fractal` | `FractalConfig` | default | Base octave stack |
| `ridge_offset` | `f32` | `1.0` | Higher values widen ridges |
| `weight_strength` | `f32` | `2.0` | Higher values emphasize persistent sharp ridges |

## Domain Warping

### `WarpConfig2`

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `amplitude` | `Vec2` | `(0.75, 0.75)` | Maximum per-axis coordinate displacement |
| `frequency` | `f32` | `1.0` | Frequency used when sampling the warp fields |
| `offset_x` | `Vec2` | `(17.3, 9.1)` | Decorrelating offset for the X warp field |
| `offset_y` | `Vec2` | `(-11.7, 23.9)` | Decorrelating offset for the Y warp field |

### `WarpConfig3`

Same semantics as `WarpConfig2`, extended to `Vec3` plus an extra `offset_z`.

## Seamless Tiling

### `TileConfig`

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `period` | `Vec2` | `(1, 1)` | Tile size in the input 2D domain before wrapping |

The seamless helper maps 2D points into a 4D torus. It therefore requires an underlying 4D-capable source.

## Domain Transforms

### `DomainTransform2`

Applied as: `rotation * (point * scale) + translation`

| Field | Type | Default |
| --- | --- | --- |
| `translation` | `Vec2` | `Vec2::ZERO` |
| `scale` | `Vec2` | `Vec2::ONE` |
| `rotation_radians` | `f32` | `0.0` |

### `DomainTransform3`

Applied as: `rotation * (point * scale) + translation`

### `DomainTransform4`

Applied as: `point * scale + translation`

## Grid Sampling

### `GridRequest2`

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `size` | `UVec2` | `256x256` | Output resolution |
| `space` | `GridSpace2` | `[-2, 2] x [-2, 2]` | Inclusive sampling bounds |

### `GridRequest3`

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `size` | `UVec3` | `48x48x32` | Output density-field resolution |
| `space` | `GridSpace3` | `[-2, 2]^3` | Inclusive sampling bounds |

## Image Output

### `NoiseImageSettings`

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `mode` | `ImageOutputMode` | `Gradient` | Grayscale or ramp-based false color |
| `normalization` | `ImageNormalization` | `Conservative` | Use observed range, published sampler range, or explicit bounds |
| `gradient` | `GradientRamp` | `terrain()` | Ignored for grayscale mode |

### `ImageNormalization`

| Variant | Meaning |
| --- | --- |
| `Observed` | Normalize from the sampled grid’s min/max. Best for debugging one grid, not for comparing presets. |
| `Conservative` | Normalize from the sampler’s published `native_range()`. Best for deterministic comparisons across runs. |
| `Explicit(Vec2)` | Normalize from an exact caller-supplied `(min, max)` pair. |

## Runtime Preview

### `NoisePreviewConfig`

| Field | Type | Notes |
| --- | --- | --- |
| `request` | `GridSampleRequest` | Current recipe, grid settings, image settings, and async/sync mode |

### `GridSampleRequest`

| Field | Type | Notes |
| --- | --- | --- |
| `recipe` | `NoiseRecipe2` | Recursive runtime sampler recipe |
| `grid` | `GridRequest2` | 2D output dimensions and bounds |
| `image` | `NoiseImageSettings` | Preview image rendering settings |
| `async_generation` | `bool` | `true` launches a background task, `false` computes inline |

### `NoiseRecipe2` and `NoiseRecipe4`

The recursive recipe enums are the config-driven bridge between pure samplers and the runtime layer.
All recipe types implement `Serialize`/`Deserialize` for RON/JSON asset loading.

- `NoiseRecipe2`: `Perlin`, `Simplex`, `Value`, `Worley`, `Fbm`, `Billow`, `Ridged`, `Warp`,
  `MultiWarp`, `Transformed`, `Tiled`
- `NoiseRecipe4`: `Perlin`, `Simplex`, `Fbm`, `Billow`, `Ridged`, `Transformed`

#### `MultiWarp` variant

Quilez-style multi-level domain warping. Each layer warps the input coordinates before the next layer:

| Field | Type | Notes |
| --- | --- | --- |
| `base` | `Box<NoiseRecipe2>` | The noise to sample at the final warped position |
| `layers` | `Vec<NoiseRecipe2>` | Warp layers applied sequentially (typically 2 for `f(p + fbm(p + fbm(p)))`) |
| `amplitude` | `f32` | Scale factor for the warp displacement |

For BRP, overlays, and logs, both enums expose `debug_stack()` so the current algorithm stack can
be summarized without source-code inspection.

### `NoiseRuntimeDiagnostics`

| Field | Type | Notes |
| --- | --- | --- |
| `active` | `bool` | Runtime systems are active on the current schedules |
| `queued_jobs` | `u64` | Total generation attempts started |
| `completed_jobs` | `u64` | Total finished generations published to `Assets<Image>` |
| `task_running` | `bool` | A background task is currently executing |
| `pending_request` | `bool` | A newer request is waiting for the current task to finish |
| `grid_size` | `UVec2` | Current preview dimensions from `NoisePreviewConfig` |
| `async_generation` | `bool` | Current request mode |
| `active_recipe` | `String` | Human-readable algorithm stack summary |
| `last_signature` | `u64` | Signature of the last published grid |
| `last_duration_ms` | `f32` | Generation time for the last published result |
| `last_min` / `last_max` | `f32` | Observed range of the last published grid |
| `last_mean` / `last_variance` | `f32` | Summary statistics for the last published grid |

### `NoiseRegenerateRequested`

| Field | Type | Notes |
| --- | --- | --- |
| `request_override` | `Option<GridSampleRequest>` | Replace the preview request before generating; `None` regenerates with the existing request |

## Builder API

`NoiseBuilder` provides a fluent interface for constructing `NoiseRecipe2` pipelines:

```rust
use saddle_procgen_noise::NoiseBuilder;

let recipe = NoiseBuilder::perlin()
    .seed(42)
    .fbm()
    .octaves(6)
    .frequency(1.5)
    .lacunarity(2.1)
    .gain(0.48)
    .warp()
    .warp_amplitude(Vec2::splat(0.9))
    .warp_frequency(1.8)
    .build();
```

Available base types: `perlin()`, `simplex()`, `value()`, `worley()`.
Fractal modes: `.fbm()`, `.billow()`, `.ridged()`.
Domain warp: `.warp()` with `.warp_amplitude()` and `.warp_frequency()`.

## Entity-Driven Pipeline

### `NoiseImageGenerator` (Component)

| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `recipe` | `NoiseRecipe2` | `Perlin(default)` | Noise recipe to generate |
| `grid` | `GridRequest2` | `256x256, [-2,2]` | Grid resolution and bounds |
| `image_settings` | `NoiseImageSettings` | default | Gradient, normalization settings |

### `NoiseImageOutput` (Component)

| Field | Type | Notes |
| --- | --- | --- |
| `handle` | `Option<Handle<Image>>` | Updated automatically when `NoiseImageGenerator` changes |
| `generation_time_ms` | f32 | Time taken for last generation |

Add `NoiseImageGenerator` to an entity and the `Pipeline` system set will automatically generate the image and insert/update `NoiseImageOutput`.

## Noise Asset Loading

Enable the `asset` feature (on by default) to load noise recipes from files:

- `.noise.ron` — RON format
- `.noise.json` — JSON format

```ron
// terrain.noise.ron
NoiseAsset(
    recipe: Fbm(
        source: Perlin(PerlinConfig(seed: NoiseSeed(42))),
        config: FractalConfig(octaves: 6, base_frequency: 1.2),
    ),
)
```

## Mask / Brush Utilities

### `stamp_noise(buffer, buffer_size, center, radius, strength, recipe, additive)`

Stamps a noise pattern onto a mutable `f32` buffer with quadratic falloff. Useful for terrain painting, sculpting, or density field modification.

### `modulate_grid(grid, recipe, grid_request)`

Multiplies each grid cell by a noise-derived factor. Useful for erosion patterns or weathering.

### `noise_mask(recipe, grid_request, threshold) -> Vec<bool>`

Creates a binary mask from a noise recipe at a given threshold.

### `generate_heightmap(recipe, grid_request) -> Vec<f32>`

Generates a normalized `[0, 1]` heightmap from any `NoiseRecipe2`. Convenience wrapper around `sample_grid2` + normalization.
