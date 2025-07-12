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
pub fn pack_4_f32_to_snorm(value: Vector4) -> u32 {
    let v = [
        quantize_snorm(value.x, 8),
        quantize_snorm(value.y, 8),
        quantize_snorm(value.z, 8),
        quantize_snorm(value.w, 8),
    ];
    (v[0] << 24) | (v[1] << 16) | (v[2] << 8) | v[3]
}

//input: [0..1] float; output: [0..255] integer
#[inline]
pub fn pack_4_f32_to_unorm(value: Vector4) -> u32 {
    let v = [
        quantize_unorm(value.x, 8),
        quantize_unorm(value.y, 8),
        quantize_unorm(value.z, 8),
        quantize_unorm(value.w, 8),
    ];
    (v[0] << 24) | (v[1] << 16) | (v[2] << 8) | v[3]
}

#[inline]
pub fn unpack_unorm_to_4_f32(value: u32) -> Vector4 {
    let r = decode_unorm((value >> 24) & 255, 8);
    let g = decode_unorm((value >> 16) & 255, 8);
    let b = decode_unorm((value >> 8) & 255, 8);
    let a = decode_unorm(value & 255, 8);
    Vector4::new(r, g, b, a)
}

#[inline]
pub fn unpack_snorm_to_4_f32(value: u32) -> Vector4 {
    let r = decode_snorm((value >> 24) & 255, 8);
    let g = decode_snorm((value >> 16) & 255, 8);
    let b = decode_snorm((value >> 8) & 255, 8);
    let a = decode_snorm(value & 255, 8);
    Vector4::new(r, g, b, a)
}

// Quantize a f32 in [0..1] range into an N-bit fixed point unorm value
// Assumes reconstruction function (q / (2^N-1)), which is the case for fixed-function normalized fixed point conversion
// Maximum reconstruction error: 1/2^(N+1)
#[inline]
pub fn quantize_unorm(mut v: f32, n: u32) -> u32 {
    let scale = ((1 << n) - 1) as f32;
    v = if v >= 0. { v } else { 0. };
    v = if v <= 1. { v } else { 1. };
    (0.5 + (v * scale)) as _
}
#[inline]
pub fn decode_unorm(i: u32, n: u32) -> f32 {
    let scale = ((1 << n) - 1) as f32;
    if i == 0 {
        0.
    } else if i == scale as _ {
        1.
    } else {
        (i as f32 - 0.5) / scale
    }
}

// Quantize a f32 in [-1..1] range into an N-bit fixed point snorm value
// Assumes reconstruction function (q / (2^(N-1)-1)), which is the case for fixed-function normalized fixed point conversion (except early OpenGL versions)
// Maximum reconstruction error: 1/2^N
#[inline]
pub fn quantize_snorm(v: f32, n: u32) -> u32 {
    let c = (1 << (n - 1)) - 1;
    let scale = c as f32;
    if v < 0. {
        return ((-v * scale) as u32 & c) | (1 << (n - 1));
    }
    (v * scale) as u32 & c
}
#[inline]
pub fn decode_snorm(i: u32, n: u32) -> f32 {
    let s = i >> (n - 1);
    let c = (1 << (n - 1)) - 1;
    let scale = c as f32;
    let r = ((i & c) as f32 + 0.5) / scale;
    if s != 0 {
        return -r;
    }
    r
}

// Quantize a f32 into half-precision floating point value (16 bit)
// Generates +-inf for overflow, preserves NaN, flushes denormals to zero, rounds to nearest
// Representable magnitude range: [6e-5; 65504]
// Maximum relative reconstruction error: 5e-4
#[inline]
pub fn quantize_half(v: f32) -> u16 {
    let x: u32 = v.to_bits();

    // Extract IEEE754 components
    let sign = x & 0x8000_0000u32;
    let exp = x & 0x7F80_0000u32;
    let man = x & 0x007F_FFFFu32;

    // Check for all exponent bits being set, which is Infinity or NaN
    if exp == 0x7F80_0000u32 {
        // Set mantissa MSB for NaN (and also keep shifted mantissa bits)
        let nan_bit = if man == 0 { 0 } else { 0x0200u32 };
        return ((sign >> 16) | 0x7C00u32 | nan_bit | (man >> 13)) as u16;
    }

    // The number is normalized, start assembling half precision version
    let half_sign = sign >> 16;
    // Unbias the exponent, then bias for half precision
    let unbiased_exp = ((exp >> 23) as i32) - 127;
    let half_exp = unbiased_exp + 15;

    // Check for exponent overflow, return +infinity
    if half_exp >= 0x1F {
        return (half_sign | 0x7C00u32) as u16;
    }

    // Check for underflow
    if half_exp <= 0 {
        // Check mantissa for what we can do
        if 14 - half_exp > 24 {
            // No rounding possibility, so this is a full underflow, return signed zero
            return half_sign as u16;
        }
        // Don't forget about hidden leading mantissa bit when assembling mantissa
        let man = man | 0x0080_0000u32;
        let mut half_man = man >> (14 - half_exp);
        // Check for rounding (see comment above functions)
        let round_bit = 1 << (13 - half_exp);
        if (man & round_bit) != 0 && (man & (3 * round_bit - 1)) != 0 {
            half_man += 1;
        }
        // No exponent for subnormals
        return (half_sign | half_man) as u16;
    }

    // Rebias the exponent
    let half_exp = (half_exp as u32) << 10;
    let half_man = man >> 13;
    // Check for rounding (see comment above functions)
    let round_bit = 0x0000_1000u32;
    if (man & round_bit) != 0 && (man & (3 * round_bit - 1)) != 0 {
        // Round it
        ((half_sign | half_exp | half_man) + 1) as u16
    } else {
        (half_sign | half_exp | half_man) as u16
    }
}

pub fn decode_half(i: u16) -> f32 {
    if i & 0x7FFFu16 == 0 {
        return f32::from_bits((i as u32) << 16);
    }

    let half_sign = (i & 0x8000u16) as u32;
    let half_exp = (i & 0x7C00u16) as u32;
    let half_man = (i & 0x03FFu16) as u32;

    // Check for an infinity or NaN when all exponent bits set
    if half_exp == 0x7C00u32 {
        // Check for signed infinity if mantissa is zero
        if half_man == 0 {
            return f32::from_bits((half_sign << 16) | 0x7F80_0000u32);
        } else {
            // NaN, keep current mantissa but also set most significiant mantissa bit
            return f32::from_bits((half_sign << 16) | 0x7FC0_0000u32 | (half_man << 13));
        }
    }

    // Calculate single-precision components with adjusted exponent
    let sign = half_sign << 16;
    // Unbias exponent
    let unbiased_exp = ((half_exp as i32) >> 10) - 15;

    // Check for subnormals, which will be normalized by adjusting exponent
    if half_exp == 0 {
        // Calculate how much to adjust the exponent by
        let e = (half_man as u16).leading_zeros() - 6;

        // Rebias and adjust exponent
        let exp = (127 - 15 - e) << 23;
        let man = (half_man << (14 + e)) & 0x7F_FF_FFu32;
        return f32::from_bits(sign | exp | man);
    }

    // Rebias exponent for a normalized normal
    let exp = ((unbiased_exp + 127) as u32) << 23;
    let man = (half_man & 0x03FFu32) << 13;
    f32::from_bits(sign | exp | man)
}

#[test]
fn encode_decode_test() {
    let max_float = 1.;
    let epsilon = 0.001;
    for _ in 0..100 {
        let v1 = crate::get_random_f32(0., max_float);
        let v2 = crate::get_random_f32(0., max_float);
        let v3 = crate::get_random_f32(0., max_float);
        let a = quantize_unorm(v1, 10);
        let b = quantize_unorm(v2, 10);
        let c = quantize_unorm(v3, 10);
        let cv1 = decode_unorm(a, 10);
        let cv2 = decode_unorm(b, 10);
        let cv3 = decode_unorm(c, 10);
        debug_assert!(
            v1 >= (cv1 - epsilon) && v1 <= (cv1 + epsilon),
            "decode a: {v1} != {cv1}"
        );
        debug_assert!(
            v2 >= (cv2 - epsilon) && v2 <= (cv2 + epsilon),
            "decode b: {v2} != {cv2}"
        );
        debug_assert!(
            v3 >= (cv3 - epsilon) && v3 <= (cv3 + epsilon),
            "decode c: {v3} != {cv3}"
        );
        let composite = (a << 20) | (b << 10) | c;
        let ca = (composite >> 20) & 0x000003FF;
        let cb = (composite >> 10) & 0x000003FF;
        let cc = composite & 0x000003FF;
        let cv1 = decode_unorm(ca, 10);
        let cv2 = decode_unorm(cb, 10);
        let cv3 = decode_unorm(cc, 10);
        debug_assert!(
            v1 >= (cv1 - epsilon) && v1 <= (cv1 + epsilon),
            "composite decode a: {v1} != {cv1}"
        );
        debug_assert!(
            v2 >= (cv2 - epsilon) && v2 <= (cv2 + epsilon),
            "composite decode b: {v2} != {cv2}"
        );
        debug_assert!(
            v3 >= (cv3 - epsilon) && v3 <= (cv3 + epsilon),
            "composite decode c: {v3} != {cv3}"
        );
    }
}

#[test]
fn encode_decode_f16_test() {
    let max_float = 1.;
    let epsilon = 0.001;
    for _ in 0..100 {
        let a = crate::get_random_f32(-max_float, max_float);
        let b = crate::get_random_f32(-max_float, max_float);
        let ca = quantize_snorm(a, 16);
        let cb = quantize_snorm(b, 16);
        let c = (ca << 16) | cb;
        let na = decode_snorm((c >> 16) & 0x0000FFFF, 16);
        let nb = decode_snorm(cb & 0x0000FFFF, 16);
        debug_assert!(
            a >= (na - epsilon) && a <= (na + epsilon),
            "composite decode a: {a} != {na}"
        );
        debug_assert!(
            b >= (nb - epsilon) && b <= (nb + epsilon),
            "composite decode b: {b} != {nb}"
        );
    }
}
