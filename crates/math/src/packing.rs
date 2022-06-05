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
    (v * 127.0 + {
        if v > 0.0 {
            0.5
        } else {
            -0.5
        }
    }) as i8
}

//input: [0..1] float; output: [0..255] integer
#[inline]
pub fn encode_f32_to_unorm(v: f32) -> u8 {
    (v * 255.0 + 0.5) as u8
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
