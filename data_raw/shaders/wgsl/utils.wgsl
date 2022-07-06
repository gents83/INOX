
fn pack_normal(normal: vec3<f32>) -> vec2<f32> {
    return vec2<f32>(normal.xy * 0.5 + 0.5);
}
fn unpack_normal(uv: vec2<f32>) -> vec3<f32> {
    return vec3<f32>(uv.xy * 2. - 1., sqrt(1.-dot(uv.xy, uv.xy)));
}

fn unpack_unorm_to_4_f32(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(((color >> 24u) / 255u) & 255u),
        f32(((color >> 16u) / 255u) & 255u),
        f32(((color >> 8u) / 255u) & 255u),
        f32((color / 255u) & 255u),
    );
}

fn hash(index: u32) -> u32 {
    var v = index;
    v = (v + 0x7ed55d16u) + (v << 12u);
    v = (v ^ 0xc761c23cu) ^ (v >> 19u);
    v = (v + 0x165667b1u) + (v << 5u);
    v = (v + 0xd3a2646cu) ^ (v << 9u);
    v = (v + 0xfd7046c5u) + (v << 3u);
    v = (v ^ 0xb55a4f09u) ^ (v >> 16u);
    return v;
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
