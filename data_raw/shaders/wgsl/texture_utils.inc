@group(2) @binding(0)
var default_sampler: sampler;

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
@group(2) @binding(1)
var texture_array: binding_array<texture_2d_array<f32>, 15>; //MAX_TEXTURE_ATLAS_COUNT
#else
@group(2) @binding(1)
var texture_1: texture_2d_array<f32>;
@group(2) @binding(2)
var texture_2: texture_2d_array<f32>;
@group(2) @binding(3)
var texture_3: texture_2d_array<f32>;
@group(2) @binding(4)
var texture_4: texture_2d_array<f32>;
@group(2) @binding(5)
var texture_5: texture_2d_array<f32>;
@group(2) @binding(6)
var texture_6: texture_2d_array<f32>;
@group(2) @binding(7)
var texture_7: texture_2d_array<f32>;
@group(2) @binding(8)
var texture_8: texture_2d_array<f32>;
@group(2) @binding(9)
var texture_9: texture_2d_array<f32>;
@group(2) @binding(10)
var texture_10: texture_2d_array<f32>;
@group(2) @binding(11)
var texture_11: texture_2d_array<f32>;
@group(2) @binding(12)
var texture_12: texture_2d_array<f32>;
@group(2) @binding(13)
var texture_13: texture_2d_array<f32>;
@group(2) @binding(14)
var texture_14: texture_2d_array<f32>;
@group(2) @binding(15)
var texture_15: texture_2d_array<f32>;
#endif


fn sample_texture(tex_coords_and_texture_index: vec3<f32>) -> vec4<f32> {
    let texture_data_index = i32(tex_coords_and_texture_index.z);
    var v = vec4<f32>(0.);
    var tex_coords = vec3<f32>(0.0, 0.0, 0.0);
    if (texture_data_index < 0) {
        return v;
    }
    let texture = &textures.data[texture_data_index];
    let atlas_index = (*texture).texture_index;
    let layer_index = i32((*texture).layer_index);

    tex_coords.x = ((*texture).area.x + tex_coords_and_texture_index.x * (*texture).area.z) / (*texture).total_width;
    tex_coords.y = ((*texture).area.y + tex_coords_and_texture_index.y * (*texture).area.w) / (*texture).total_height;
    tex_coords.z = f32(layer_index);

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
    v = textureSampleLevel(texture_array[atlas_index], default_sampler, tex_coords.xy, layer_index, 0.);
#else
    switch (atlas_index) {
        case 0u: { v = textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 1u: { v = textureSampleLevel(texture_2, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 2u: { v = textureSampleLevel(texture_3, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 3u: { v = textureSampleLevel(texture_4, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 4u: { v = textureSampleLevel(texture_5, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 5u: { v = textureSampleLevel(texture_6, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 6u: { v = textureSampleLevel(texture_7, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 7u: { v = textureSampleLevel(texture_8, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 9u: { v = textureSampleLevel(texture_9, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 10u: { v = textureSampleLevel(texture_10, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 11u: { v = textureSampleLevel(texture_11, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 12u: { v = textureSampleLevel(texture_12, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 13u: { v = textureSampleLevel(texture_13, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 14u: { v = textureSampleLevel(texture_14, default_sampler, tex_coords.xy, layer_index, 0.); }
        case 15u: { v = textureSampleLevel(texture_15, default_sampler, tex_coords.xy, layer_index, 0.); }
        default { v = textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, 0.); }
    };
#endif
    return v;
}


fn get_uv(uv_set: vec4<u32>, texture_index: u32, coords_set: u32) -> vec3<f32> {
    var uv = vec2<f32>(0., 0.);
    switch (coords_set) {
        case 1u: { uv = unpack2x16float(uv_set.y); }
        case 2u: { uv = unpack2x16float(uv_set.z); }
        case 3u: { uv = unpack2x16float(uv_set.w); }
        default { uv = unpack2x16float(uv_set.x); }
    }
    return vec3<f32>(uv, f32(texture_index));
}

