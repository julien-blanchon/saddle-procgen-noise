#[must_use]
pub fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[must_use]
pub fn signed_to_unit(value: f32) -> f32 {
    clamp_unit(value * 0.5 + 0.5)
}

#[must_use]
pub fn unit_to_signed(value: f32) -> f32 {
    clamp_unit(value) * 2.0 - 1.0
}

#[must_use]
pub fn remap_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let span = (in_max - in_min).max(f32::EPSILON);
    let t = (value - in_min) / span;
    out_min + (out_max - out_min) * t
}

#[must_use]
pub fn remap_clamped(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let span = (in_max - in_min).max(f32::EPSILON);
    let t = ((value - in_min) / span).clamp(0.0, 1.0);
    out_min + (out_max - out_min) * t
}

#[must_use]
pub fn bias(value: f32, bias: f32) -> f32 {
    let value = clamp_unit(value);
    let bias = bias.clamp(0.0001, 0.9999);
    value / (((1.0 / bias) - 2.0) * (1.0 - value) + 1.0)
}

#[must_use]
pub fn gain(value: f32, gain: f32) -> f32 {
    let value = clamp_unit(value);
    if value < 0.5 {
        bias(value * 2.0, gain) * 0.5
    } else {
        1.0 - bias(2.0 - value * 2.0, gain) * 0.5
    }
}

#[must_use]
pub fn contrast_pow(value: f32, exponent: f32) -> f32 {
    clamp_unit(value).powf(exponent.max(0.0001))
}

#[must_use]
pub fn binary_threshold(value: f32, threshold: f32) -> bool {
    value >= threshold
}

#[must_use]
pub fn smoothstep_threshold(value: f32, low: f32, high: f32) -> f32 {
    let t = ((value - low) / (high - low).max(f32::EPSILON)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
