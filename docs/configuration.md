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

- `NoiseRecipe2`: `Perlin`, `Simplex`, `Worley`, `Fbm`, `Billow`, `Ridged`, `Warp`,
  `Transformed`, `Tiled`
- `NoiseRecipe4`: `Perlin`, `Simplex`, `Fbm`, `Billow`, `Ridged`, `Transformed`

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
