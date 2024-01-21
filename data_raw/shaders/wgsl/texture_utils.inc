@group(2) @binding(0)
var default_sampler: sampler;

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
@group(2) @binding(1)
var texture_array: binding_array<texture_2d_array<f32>, 8>; //MAX_TEXTURE_ATLAS_COUNT
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
#endif


fn sample_texture(tex_coords_and_texture_index: vec3<f32>) -> vec4<f32> {
    let texture_data_index = i32(tex_coords_and_texture_index.z);
    var v = vec4<f32>(0.);
    var tex_coords = vec3<f32>(0.0, 0.0, 0.0);
    if (texture_data_index < 0) {
        return v;
    }
    let texture = &textures.data[texture_data_index];
    var texture_index = (*texture).texture_and_layer_index;
    let min = unpack2x16float((*texture).min);
    let max = unpack2x16float((*texture).max);
    let size = unpack2x16float((*texture).size);
    if (texture_index < 0) {
        texture_index *= -1;
    } 
    let atlas_index = u32(texture_index >> 3);
    let layer_index = i32(texture_index &  0x00000007);

    tex_coords.x = (f32(min.x) + mod_f32(tex_coords_and_texture_index.x, 1.) * f32(max.x)) / f32(size.x);
    tex_coords.y = (f32(min.y) + mod_f32(tex_coords_and_texture_index.y, 1.) * f32(max.y)) / f32(size.y);
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
        default { v = textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, 0.); }
    };
#endif
    return v;
}
