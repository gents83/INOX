@group(2) @binding(0)
var default_sampler: sampler;
@group(2) @binding(1)
var unfiltered_sampler: sampler;
@group(2) @binding(2)
var depth_sampler: sampler_comparison;

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
@group(2) @binding(3)
var texture_array: binding_array<texture_2d_array<f32>, 16>; //MAX_TEXTURE_ATLAS_COUNT
#else
@group(2) @binding(3)
var texture_1: texture_2d_array<f32>;
@group(2) @binding(4)
var texture_2: texture_2d_array<f32>;
@group(2) @binding(5)
var texture_3: texture_2d_array<f32>;
@group(2) @binding(6)
var texture_4: texture_2d_array<f32>;
@group(2) @binding(7)
var texture_5: texture_2d_array<f32>;
@group(2) @binding(8)
var texture_6: texture_2d_array<f32>;
@group(2) @binding(9)
var texture_7: texture_2d_array<f32>;
@group(2) @binding(10)
var texture_8: texture_2d_array<f32>;
@group(2) @binding(11)
var texture_9: texture_2d_array<f32>;
@group(2) @binding(12)
var texture_10: texture_2d_array<f32>;
@group(2) @binding(13)
var texture_11: texture_2d_array<f32>;
@group(2) @binding(14)
var texture_12: texture_2d_array<f32>;
@group(2) @binding(15)
var texture_13: texture_2d_array<f32>;
@group(2) @binding(16)
var texture_14: texture_2d_array<f32>;
@group(2) @binding(17)
var texture_15: texture_2d_array<f32>;
@group(2) @binding(18)
var texture_16: texture_2d_array<f32>;
#endif


fn sample_texture(tex_coords_and_texture_index: vec3<f32>) -> vec4<f32> {
    let texture_data_index = i32(tex_coords_and_texture_index.z);
    var tex_coords = vec3<f32>(0.0, 0.0, 0.0);
    if (texture_data_index < 0) {
        return vec4<f32>(tex_coords, 0.);
    }
    let texture = &textures.data[texture_data_index];
    let atlas_index = (*texture).texture_index;
    let layer_index = i32((*texture).layer_index);

    tex_coords.x = ((*texture).area.x + tex_coords_and_texture_index.x * (*texture).area.z) / (*texture).total_width;
    tex_coords.y = ((*texture).area.y + tex_coords_and_texture_index.y * (*texture).area.w) / (*texture).total_height;
    tex_coords.z = f32(layer_index);

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
    return textureSampleLevel(texture_array[atlas_index], default_sampler, tex_coords.xy, layer_index, tex_coords.z);
#else
    switch (atlas_index) {
        default { return textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 1u: { return textureSampleLevel(texture_2, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 2u: { return textureSampleLevel(texture_3, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 3u: { return textureSampleLevel(texture_4, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 4u: { return textureSampleLevel(texture_5, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 5u: { return textureSampleLevel(texture_6, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 6u: { return textureSampleLevel(texture_7, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 7u: { return textureSampleLevel(texture_8, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 8u: { return textureSampleLevel(texture_9, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 9u: { return textureSampleLevel(texture_10, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 10u: { return textureSampleLevel(texture_11, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 11u: { return textureSampleLevel(texture_12, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 12u: { return textureSampleLevel(texture_13, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 13u: { return textureSampleLevel(texture_14, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 14u: { return textureSampleLevel(texture_15, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
        case 15u: { return textureSampleLevel(texture_16, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }
    }
    return textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
#endif
}



fn load_texture(tex_coords_and_texture_index: vec3<i32>) -> vec4<f32> {
    let atlas_index = tex_coords_and_texture_index.z;
    let layer_index = 0;

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
    return textureLoad(texture_array[atlas_index], tex_coords_and_texture_index.xy, layer_index, layer_index);
#else
    switch (atlas_index) {
        default { return textureLoad(texture_1, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 1: { return textureLoad(texture_2, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 2: { return textureLoad(texture_3, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 3: { return textureLoad(texture_4, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 4: { return textureLoad(texture_5, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 5: { return textureLoad(texture_6, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 6: { return textureLoad(texture_7, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 7: { return textureLoad(texture_8, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 8: { return textureLoad(texture_9, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 9: { return textureLoad(texture_10, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 10: { return textureLoad(texture_11, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 11: { return textureLoad(texture_12, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 12: { return textureLoad(texture_13, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 13: { return textureLoad(texture_14, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 14: { return textureLoad(texture_15, tex_coords_and_texture_index.xy, layer_index, layer_index); }
        case 15: { return textureLoad(texture_16, tex_coords_and_texture_index.xy, layer_index, layer_index); }
    }
#endif
}

fn get_uv(uvs: vec4<f32>, texture_index: u32, coords_set: u32) -> vec3<f32> {
    //var uv = unpack2x16float(u32(uvs.x));
    //if (coords_set == 1u) {
    //    uv = unpack2x16float(u32(uvs.y));
    //} else if (coords_set == 2u) {
    //    uv = unpack2x16float(u32(uvs.z));
    //} else if (coords_set == 3u) {
    //    uv = unpack2x16float(u32(uvs.w));
    //}
    var uv = uvs.xy;
    if (coords_set == 1u) {
        uv = uvs.zw;
    }
    return vec3<f32>(uv, f32(texture_index));
}

fn load(texture_index: u32, v: vec2<i32>) -> vec4<f32> {  
    return load_texture(vec3<i32>(v.xy, i32(texture_index)));
}
