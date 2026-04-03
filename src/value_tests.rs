use super::*;
use crate::{
    config::ValueConfig,
    hash::{hash2, hash3, hash4},
    sample::{NoiseSource, RangeSemantics},
    NoiseSeed,
};

const EPSILON: f32 = 1.0e-6;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn lattice_points_match_their_hashed_corner_values() {
    let source = Value::new(NoiseSeed(13));

    assert_close(
        source.sample(Vec2::new(2.0, -3.0)),
        lattice_value(hash2(source.seed, 2, -3)),
    );
    assert_close(
        source.sample(Vec3::new(2.0, -3.0, 4.0)),
        lattice_value(hash3(source.seed, 2, -3, 4)),
    );
    assert_close(
        source.sample(Vec4::new(2.0, -3.0, 4.0, -1.0)),
        lattice_value(hash4(source.seed, 2, -3, 4, -1)),
    );
}

#[test]
fn value_noise_is_seed_sensitive_and_stays_in_range() {
    let first = Value::from(ValueConfig {
        seed: NoiseSeed::new(7),
    });
    let second = Value::from(ValueConfig {
        seed: NoiseSeed::new(8),
    });

    let sample_a = first.sample(Vec2::new(0.33, -1.7));
    let sample_b = first.sample(Vec2::new(0.33, -1.7));
    let sample_c = second.sample(Vec2::new(0.33, -1.7));

    assert_close(sample_a, sample_b);
    assert_ne!(sample_a.to_bits(), sample_c.to_bits());

    for x in -2..=2 {
        for y in -2..=2 {
            let point = Vec2::new(x as f32 * 0.37, y as f32 * -0.41);
            let value = first.sample(point);
            assert!(
                (-1.0..=1.0).contains(&value),
                "value sample {value} was out of range"
            );
        }
    }

    let range = <Value as NoiseSource<Vec2>>::native_range(&first);
    assert_eq!(range.min, -1.0);
    assert_eq!(range.max, 1.0);
    assert_eq!(range.semantics, RangeSemantics::Strict);
}
