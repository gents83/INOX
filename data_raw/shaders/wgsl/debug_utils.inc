
fn debug_color_override(color: vec4<f32>, pixel: vec2<u32>) -> vec4<f32> {
    var out_color = color;
    if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS) != 0) {
        let visibility_output = textureLoad(visibility_texture, pixel, 0);
        let visibility_id = pack4x8unorm(visibility_output);
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            let meshlet_id = (visibility_id >> 8u); 
            let meshlet_color = hash(meshlet_id + 1u);
            out_color = vec4<f32>(vec3<f32>(
                f32(meshlet_color & 255u),
                f32((meshlet_color >> 8u) & 255u),
                f32((meshlet_color >> 16u) & 255u)
            ) / 255., 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_VISIBILITY_BUFFER) != 0) {
        out_color = textureLoad(visibility_texture, pixel, 0);
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER) != 0) {
        let depth = textureLoad(depth_texture, pixel, 0);
        let v = vec3<f32>(1. - depth);
        out_color = vec4<f32>(v, 1.);
    } 
    return out_color;
}