
fn unpack_unorm_to_4_f32(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(((color >> 24u) / 255u) & 255u),
        f32(((color >> 16u) / 255u) & 255u),
        f32(((color >> 8u) / 255u) & 255u),
        f32((color / 255u) & 255u),
    );
}
