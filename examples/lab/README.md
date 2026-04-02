# Noise Lab

Crate-local standalone lab for validating the shared `noise` crate through its pure APIs, thin Bevy runtime plugin, BRP inspection, and crate-local E2E scenarios.

## Purpose

- compare multiple noise families side by side
- prove seamless tiling visually
- exercise async regeneration through `NoisePlugin`
- expose BRP-queryable diagnostics and preview config resources for live tuning

## Status

Working

## Run

```bash
cargo run -p noise_lab
```

Controls:

- `1`: async preview view
- `2`: preset comparison grid
- `3`: seamless tiling view
- `Q`: async preview uses Perlin
- `W`: async preview uses Simplex
- `E`: async preview uses Worley
- `R`: async preview uses FBM
- `T`: async preview uses Ridged
- `Y`: async preview uses Domain Warp
- `Space`: regenerate the async preview with a fresh seed

## E2E

```bash
cargo run -p noise_lab --features e2e -- noise_smoke
cargo run -p noise_lab --features e2e -- noise_presets_compare
cargo run -p noise_lab --features e2e -- noise_async_regen
```

## BRP

If the helper launcher is unavailable, run `cargo run -p noise_lab` in one terminal and use the BRP commands below from another terminal.

```bash
uv run --active --project .codex/skills/bevy-brp/script brp app launch noise_lab
uv run --active --project .codex/skills/bevy-brp/script brp resource list
uv run --active --project .codex/skills/bevy-brp/script brp resource get noise_lab::LabDiagnostics
uv run --active --project .codex/skills/bevy-brp/script brp resource get noise::components::NoisePreviewConfig
uv run --active --project .codex/skills/bevy-brp/script brp resource get noise::components::NoiseRuntimeDiagnostics
uv run --active --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/noise_lab.png
uv run --active --project .codex/skills/bevy-brp/script brp extras shutdown
```
