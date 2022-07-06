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
        case 0: return textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 1: return textureSampleLevel(texture_2, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 2: return textureSampleLevel(texture_3, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 3: return textureSampleLevel(texture_4, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 4: return textureSampleLevel(texture_5, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 5: return textureSampleLevel(texture_6, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 6: return textureSampleLevel(texture_7, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 7: return textureSampleLevel(texture_8, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 8: return textureSampleLevel(texture_9, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 9: return textureSampleLevel(texture_10, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 10: return textureSampleLevel(texture_11, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 11: return textureSampleLevel(texture_12, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 12: return textureSampleLevel(texture_13, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 13: return textureSampleLevel(texture_14, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 14: return textureSampleLevel(texture_15, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
        case 15: return textureSampleLevel(texture_16, default_sampler, tex_coords.xy, layer_index, tex_coords.z);;
    }
    return textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
#endif
}
