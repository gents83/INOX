
fn unpack_unorm_to_4_f32(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(((color >> 24u) / 255u) & 255u),
        f32(((color >> 16u) / 255u) & 255u),
        f32(((color >> 8u) / 255u) & 255u),
        f32((color / 255u) & 255u),
    );
}

// 0-1 from 0-255
fn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(10.31475);
    let lower = srgb / vec3<f32>(3294.6);
    let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

// [u8; 4] SRGB as u32 -> [r, g, b, a]
fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(color & 255u),
        f32((color >> 8u) & 255u),
        f32((color >> 16u) & 255u),
        f32((color >> 24u) & 255u),
    );
}
