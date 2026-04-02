use crate::NoiseSeed;

#[inline]
fn mix32(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^= value >> 16;
    value
}

#[inline]
fn combine(seed: NoiseSeed, coords: &[u32]) -> u32 {
    let mut hash = mix32(seed.0 ^ 0xA511_E9B3);
    for (index, coord) in coords.iter().copied().enumerate() {
        hash = mix32(
            hash ^ coord
                ^ (index as u32)
                    .wrapping_mul(0x9E37_79B9)
                    .rotate_left((index as u32) & 15),
        );
    }
    hash
}

#[inline]
pub fn hash2(seed: NoiseSeed, x: i32, y: i32) -> u32 {
    combine(seed, &[x as u32, y as u32])
}

#[inline]
pub fn hash3(seed: NoiseSeed, x: i32, y: i32, z: i32) -> u32 {
    combine(seed, &[x as u32, y as u32, z as u32])
}

#[inline]
pub fn hash4(seed: NoiseSeed, x: i32, y: i32, z: i32, w: i32) -> u32 {
    combine(seed, &[x as u32, y as u32, z as u32, w as u32])
}

#[inline]
pub fn unit_float(hash: u32) -> f32 {
    hash as f32 / u32::MAX as f32
}
