use super::*;
use proptest::prelude::*;

const EPSILON: f32 = 1.0e-5;

fn assert_close(actual: f32, expected: f32, epsilon: f32) {
    assert!(
        (actual - expected).abs() <= epsilon,
        "expected {expected}, got {actual} (epsilon {epsilon})"
    );
}

#[test]
fn fade_curve_has_expected_endpoints_and_symmetry_midpoint() {
    assert_close(fade_curve(0.0), 0.0, EPSILON);
    assert_close(fade_curve(0.5), 0.5, EPSILON);
    assert_close(fade_curve(1.0), 1.0, EPSILON);
}

#[test]
fn canonical_reference_samples_stay_stable() {
    let point2 = Vec2::new(0.125, 0.25);
    let point3 = Vec3::new(0.125, 0.25, -0.5);
    let point4 = Vec4::new(0.125, 0.25, -0.5, 0.75);

    assert_close(
        Perlin::new(NoiseSeed(7)).sample(point2),
        0.270_650_5,
        1.0e-6,
    );
    assert_close(
        Simplex::new(NoiseSeed(11)).sample(point2),
        -0.526_815_83,
        1.0e-6,
    );
    assert_close(
        Worley::new(WorleyConfig {
            seed: NoiseSeed(19),
            return_type: WorleyReturnType::F2MinusF1,
            ..default()
        })
        .sample(point2),
        0.035_742_67,
        1.0e-6,
    );
    assert_close(
        Perlin::new(NoiseSeed(23)).sample(point3),
        0.273_480_9,
        1.0e-6,
    );
    assert_close(
        Simplex::new(NoiseSeed(29)).sample(point3),
        -0.580_505_85,
        1.0e-6,
    );
    assert_close(
        Perlin::new(NoiseSeed(31)).sample(point4),
        0.030_215_055,
        1.0e-6,
    );
    assert_close(
        Simplex::new(NoiseSeed(37)).sample(point4),
        -0.020_284_727,
        1.0e-6,
    );
}

#[test]
fn primitive_samplers_are_deterministic_and_seed_sensitive() {
    let point = Vec2::new(0.3125, -0.875);

    let perlin_a = Perlin::new(NoiseSeed(7));
    let perlin_b = Perlin::new(NoiseSeed(7));
    let perlin_c = Perlin::new(NoiseSeed(8));
    assert_close(perlin_a.sample(point), perlin_b.sample(point), EPSILON);
    assert_ne!(
        perlin_a.sample(point).to_bits(),
        perlin_c.sample(point).to_bits()
    );

    let simplex_a = Simplex::new(NoiseSeed(11));
    let simplex_b = Simplex::new(NoiseSeed(11));
    let simplex_c = Simplex::new(NoiseSeed(12));
    assert_close(simplex_a.sample(point), simplex_b.sample(point), EPSILON);
    assert_ne!(
        simplex_a.sample(point).to_bits(),
        simplex_c.sample(point).to_bits()
    );

    let worley_a = Worley::new(WorleyConfig {
        seed: NoiseSeed(19),
        return_type: WorleyReturnType::F2MinusF1,
        ..default()
    });
    let worley_b = Worley::new(WorleyConfig {
        seed: NoiseSeed(19),
        return_type: WorleyReturnType::F2MinusF1,
        ..default()
    });
    let worley_c = Worley::new(WorleyConfig {
        seed: NoiseSeed(20),
        return_type: WorleyReturnType::F2MinusF1,
        ..default()
    });
    assert_close(worley_a.sample(point), worley_b.sample(point), EPSILON);
    assert_ne!(
        worley_a.sample(point).to_bits(),
        worley_c.sample(point).to_bits()
    );
}

#[test]
fn worley_return_modes_are_ordered() {
    let point = Vec2::new(1.125, -0.375);
    let config = WorleyConfig {
        seed: NoiseSeed(31),
        jitter: 0.9,
        ..default()
    };

    let f1 = Worley::new(WorleyConfig {
        return_type: WorleyReturnType::F1,
        ..config
    })
    .sample(point);
    let f2 = Worley::new(WorleyConfig {
        return_type: WorleyReturnType::F2,
        ..config
    })
    .sample(point);
    let diff = Worley::new(WorleyConfig {
        return_type: WorleyReturnType::F2MinusF1,
        ..config
    })
    .sample(point);

    assert!(f1 <= f2, "expected F1 <= F2, got {f1} > {f2}");
    assert_close(diff, (f2 - f1).max(0.0), 1.0e-4);
}

#[test]
fn tiled_noise_matches_on_opposite_edges() {
    let tiled = Tiled2::new(
        NoiseRecipe4::Fbm {
            source: Box::new(NoiseRecipe4::Simplex(SimplexConfig {
                seed: NoiseSeed(44),
            })),
            config: FractalConfig {
                octaves: 4,
                base_frequency: 1.0,
                ..default()
            },
        },
        TileConfig {
            period: Vec2::splat(1.0),
        },
    );

    for step in 0..17 {
        let t = step as f32 / 16.0;
        assert_close(
            tiled.sample(Vec2::new(0.0, t)),
            tiled.sample(Vec2::new(1.0, t)),
            2.0e-4,
        );
        assert_close(
            tiled.sample(Vec2::new(t, 0.0)),
            tiled.sample(Vec2::new(t, 1.0)),
            2.0e-4,
        );
    }
}

#[test]
fn batch_sampling_matches_direct_sampling() {
    let recipe = NoiseRecipe2::Warp {
        base: Box::new(NoiseRecipe2::Fbm {
            source: Box::new(NoiseRecipe2::Perlin(PerlinConfig {
                seed: NoiseSeed(61),
            })),
            config: FractalConfig {
                octaves: 4,
                base_frequency: 1.15,
                ..default()
            },
        }),
        warp_x: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
            seed: NoiseSeed(62),
        })),
        warp_y: Box::new(NoiseRecipe2::Simplex(SimplexConfig {
            seed: NoiseSeed(63),
        })),
        config: WarpConfig2 {
            amplitude: Vec2::splat(0.8),
            frequency: 1.6,
            ..default()
        },
    };
    let request = GridRequest2 {
        size: UVec2::new(12, 9),
        space: GridSpace2 {
            min: Vec2::new(-1.5, -0.5),
            max: Vec2::new(2.0, 1.25),
        },
    };

    let grid = sample_grid2(&recipe, &request);
    for y in 0..request.size.y {
        for x in 0..request.size.x {
            let point = request.space.sample_position(x, y, request.size);
            let index = (y * request.size.x + x) as usize;
            assert_close(grid.values[index], recipe.sample(point), EPSILON);
        }
    }
}

#[test]
fn independent_chunk_edges_align() {
    let source = Simplex::new(NoiseSeed(77));
    let left = sample_grid2(
        &source,
        &GridRequest2 {
            size: UVec2::new(10, 10),
            space: GridSpace2 {
                min: Vec2::new(0.0, -1.0),
                max: Vec2::new(1.0, 1.0),
            },
        },
    );
    let right = sample_grid2(
        &source,
        &GridRequest2 {
            size: UVec2::new(10, 10),
            space: GridSpace2 {
                min: Vec2::new(1.0, -1.0),
                max: Vec2::new(2.0, 1.0),
            },
        },
    );

    let width = left.size.x as usize;
    for y in 0..left.size.y as usize {
        let left_edge = left.values[y * width + width - 1];
        let right_edge = right.values[y * width];
        assert_close(left_edge, right_edge, EPSILON);
    }
}

#[test]
fn image_helpers_preserve_dimensions_and_rgba_layout() {
    let grid = Grid2 {
        size: UVec2::new(2, 2),
        values: vec![-1.0, 0.0, 0.5, 1.0],
        stats: GridStats {
            min: -1.0,
            max: 1.0,
            mean: 0.125,
            variance: 0.546_875,
        },
    };

    let gray = grid_to_grayscale_image(
        &grid,
        Some(NoiseRange::new(-1.0, 1.0, RangeSemantics::Strict)),
    );
    assert_eq!(gray.texture_descriptor.size.width, 2);
    assert_eq!(gray.texture_descriptor.size.height, 2);
    assert_eq!(gray.data.as_ref().map(Vec::len), Some(16));
    assert_eq!(gray.data.as_ref().unwrap()[..4], [0, 0, 0, 255]);

    let gradient = grid_to_gradient_image(
        &grid,
        &NoiseImageSettings {
            gradient: GradientRamp::heatmap(),
            ..default()
        },
        Some(NoiseRange::new(-1.0, 1.0, RangeSemantics::Strict)),
    );
    assert_eq!(gradient.texture_descriptor.size.width, 2);
    assert_eq!(gradient.texture_descriptor.size.height, 2);
    assert_eq!(gradient.data.as_ref().map(Vec::len), Some(16));
}

#[test]
fn normalization_helpers_match_documented_clamped_ranges() {
    let range = NoiseRange::new(-2.0, 2.0, RangeSemantics::Strict);
    assert_close(range.normalize_clamped(-2.0), 0.0, EPSILON);
    assert_close(range.normalize_clamped(0.0), 0.5, EPSILON);
    assert_close(range.normalize_clamped(2.0), 1.0, EPSILON);
    assert_close(range.signed_clamped(-2.0), -1.0, EPSILON);
    assert_close(range.signed_clamped(0.0), 0.0, EPSILON);
    assert_close(range.signed_clamped(2.0), 1.0, EPSILON);
}

#[test]
fn domain_warp_and_fractal_outputs_stay_finite() {
    let fbm = Fbm::new(
        Perlin::new(NoiseSeed(90)),
        FractalConfig {
            octaves: 5,
            base_frequency: 1.35,
            ..default()
        },
    );
    let ridged = Ridged::new(
        Simplex::new(NoiseSeed(91)),
        RidgedConfig {
            fractal: FractalConfig {
                octaves: 5,
                base_frequency: 1.7,
                ..default()
            },
            ..default()
        },
    );
    let warped = DomainWarp2::new(
        fbm,
        Simplex::new(NoiseSeed(92)),
        Simplex::new(NoiseSeed(93)),
        WarpConfig2 {
            amplitude: Vec2::splat(0.85),
            frequency: 1.8,
            ..default()
        },
    );

    for y in -6..=6 {
        for x in -6..=6 {
            let point = Vec2::new(x as f32 * 0.21, y as f32 * 0.17);
            assert!(warped.sample(point).is_finite());
            assert!(ridged.sample(point).is_finite());
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn primitive_noise_stays_finite_over_representative_inputs(
        x in -32.0f32..32.0,
        y in -32.0f32..32.0,
        z in -16.0f32..16.0,
    ) {
        let point2 = Vec2::new(x, y);
        let point3 = Vec3::new(x, y, z);
        let worley = Worley::new(WorleyConfig {
            seed: NoiseSeed(103),
            return_type: WorleyReturnType::F2MinusF1,
            ..default()
        });

        prop_assert!(Perlin::new(NoiseSeed(101)).sample(point2).is_finite());
        prop_assert!(Simplex::new(NoiseSeed(102)).sample(point2).is_finite());
        prop_assert!(worley.sample(point2).is_finite());
        prop_assert!(Perlin::new(NoiseSeed(104)).sample(point3).is_finite());
        prop_assert!(Simplex::new(NoiseSeed(105)).sample(point3).is_finite());
    }

    #[test]
    fn gradient_noise_changes_smoothly_for_small_coordinate_deltas(
        x in -4.0f32..4.0,
        y in -4.0f32..4.0,
    ) {
        let point = Vec2::new(x, y);
        let delta = Vec2::splat(0.001);

        let perlin_a = Perlin::new(NoiseSeed(201)).sample(point);
        let perlin_b = Perlin::new(NoiseSeed(201)).sample(point + delta);
        prop_assert!((perlin_a - perlin_b).abs() < 0.05);

        let simplex_a = Simplex::new(NoiseSeed(202)).sample(point);
        let simplex_b = Simplex::new(NoiseSeed(202)).sample(point + delta);
        prop_assert!((simplex_a - simplex_b).abs() < 0.08);
    }
}
