use crate::Vector4;

// https://docs.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-data-conversion

//output: [-1..1] float in range [min..max]
#[inline]
pub fn normalize_f32_in_neg1_pos1(v: f32, min: f32, max: f32) -> f32 {
    2. * (v - min) / (max - min) - 1.
}

//output: [0..1] float in range [min..max]
#[inline]
pub fn normalize_f32_in_0_1(v: f32, min: f32, max: f32) -> f32 {
    (v - min) / (max - min)
}

//input: [-1..1] float; output: [-127..127] integer
#[inline]
pub fn encode_f32_to_snorm(v: f32) -> i8 {
    quantize_snorm(v, 8) as _
}

//input: [0..1] float; output: [0..255] integer
#[inline]
pub fn encode_f32_to_unorm(v: f32) -> u8 {
    quantize_unorm(v, 8) as _
}

//input: [-1..1] float; output: [-127..127] integer
#[inline]
pub fn pack_4_f32_to_snorm(value: Vector4) -> i32 {
    let v = [
        encode_f32_to_snorm(value.x),
        encode_f32_to_snorm(value.y),
        encode_f32_to_snorm(value.z),
        encode_f32_to_snorm(value.w),
    ];
    ((v[0] as i32) << 24) | ((v[1] as i32) << 16) | ((v[2] as i32) << 8) | v[3] as i32
}

//input: [0..1] float; output: [0..255] integer
#[inline]
pub fn pack_4_f32_to_unorm(value: Vector4) -> u32 {
    let v = [
        encode_f32_to_unorm(value.x),
        encode_f32_to_unorm(value.y),
        encode_f32_to_unorm(value.z),
        encode_f32_to_unorm(value.w),
    ];
    ((v[0] as u32) << 24) | ((v[1] as u32) << 16) | ((v[2] as u32) << 8) | v[3] as u32
}

// Quantize a f32 in [0..1] range into an N-bit fixed point unorm value
// Assumes reconstruction function (q / (2^N-1)), which is the case for fixed-function normalized fixed point conversion
// Maximum reconstruction error: 1/2^(N+1)
#[inline]
pub fn quantize_unorm(mut v: f32, n: u32) -> i32 {
    let scale = ((1 << n) - 1) as f32;
    v = if v >= 0. { v } else { 0. };
    v = if v <= 1. { v } else { 1. };
    (v * scale + 0.5) as _
}

// Quantize a f32 in [-1..1] range into an N-bit fixed point snorm value
// Assumes reconstruction function (q / (2^(N-1)-1)), which is the case for fixed-function normalized fixed point conversion (except early OpenGL versions)
// Maximum reconstruction error: 1/2^N
#[inline]
pub fn quantize_snorm(mut v: f32, n: u32) -> i32 {
    let scale = ((1 << (n - 1)) - 1) as f32;
    let round = if v >= 0. { 0.5 } else { -0.5 };
    v = if v >= -1. { v } else { -1. };
    v = if v <= 1. { v } else { 1. };
    (v * scale + round) as _
}

// Quantize a f32 into half-precision floating point value (16 bit)
// Generates +-inf for overflow, preserves NaN, flushes denormals to zero, rounds to nearest
// Representable magnitude range: [6e-5; 65504]
// Maximum relative reconstruction error: 5e-4
#[inline]
pub fn quantize_half(v: f32) -> u16 {
    union UnionFloat {
        f: f32,
        ui: u32,
    }
    let u = UnionFloat { f: v };
    let ui = unsafe { u.ui };

    let s = (ui >> 16) & 0x8000;
    let em = ui & 0x7fffffff;

    /* bias exponent and round to nearest; 112 is relative exponent bias (127-15) */
    let mut h = (em - (112 << 23) + (1 << 12)) >> 13;

    /* underflow: flush to zero; 113 encodes exponent -14 */
    h = if em < (113 << 23) { 0 } else { h };

    /* overflow: infinity; 143 encodes exponent 16 */
    h = if em >= (143 << 23) { 0x7c00 } else { h };

    /* NaN; note that we convert all types of NaN to qNaN */
    h = if em > (255 << 23) { 0x7e00 } else { h };

    (s | h) as _
}
