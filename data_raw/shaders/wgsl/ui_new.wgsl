#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> positions_and_colors: PositionsAndColors;
@group(0) @binding(2)
var<storage, read> uvs_0: UVs;
@group(0) @binding(3)
var<storage, read> meshes: Meshes;
@group(0) @binding(4)
var<storage, read> materials: Materials;
@group(0) @binding(5)
var<storage, read> textures: Textures;



@group(1) @binding(0)
var default_sampler: sampler;
@group(1) @binding(1)
var depth_sampler: sampler_comparison;

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
@group(1) @binding(2)
var texture_array: binding_array<texture_2d_array<f32>, 16>; //MAX_TEXTURE_ATLAS_COUNT
#else
@group(1) @binding(2)
var texture_1: texture_2d_array<f32>;
@group(1) @binding(3)
var texture_2: texture_2d_array<f32>;
@group(1) @binding(4)
var texture_3: texture_2d_array<f32>;
@group(1) @binding(5)
var texture_4: texture_2d_array<f32>;
@group(1) @binding(6)
var texture_5: texture_2d_array<f32>;
@group(1) @binding(7)
var texture_6: texture_2d_array<f32>;
@group(1) @binding(8)
var texture_7: texture_2d_array<f32>;
@group(1) @binding(9)
var texture_8: texture_2d_array<f32>;
@group(1) @binding(10)
var texture_9: texture_2d_array<f32>;
@group(1) @binding(11)
var texture_10: texture_2d_array<f32>;
@group(1) @binding(12)
var texture_11: texture_2d_array<f32>;
@group(1) @binding(13)
var texture_12: texture_2d_array<f32>;
@group(1) @binding(14)
var texture_13: texture_2d_array<f32>;
@group(1) @binding(15)
var texture_14: texture_2d_array<f32>;
@group(1) @binding(16)
var texture_15: texture_2d_array<f32>;
@group(1) @binding(17)
var texture_16: texture_2d_array<f32>;
#endif


fn get_texture_color(material_index: u32, texture_type: u32, vertex_index: u32) -> vec4<f32> {
    let texture_data_index = materials.data[material_index].textures_indices[texture_type];
    var tex_coords = vec3<f32>(0.0, 0.0, 0.0);
    if (texture_data_index < 0) {
        return vec4<f32>(tex_coords, 0.);
    }
    let atlas_index = textures.data[texture_data_index].texture_index;
    let layer_index = i32(textures.data[texture_data_index].layer_index);

    tex_coords.x = (textures.data[texture_data_index].area.x + 0.5 + uvs_0.data[vertex_index].x * textures.data[texture_data_index].area.z) / textures.data[texture_data_index].total_width;
    tex_coords.y = (textures.data[texture_data_index].area.y + 0.5 + uvs_0.data[vertex_index].y * textures.data[texture_data_index].area.w) / textures.data[texture_data_index].total_height;
    tex_coords.z = f32(textures.data[texture_data_index].layer_index);

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
    return textureSampleLevel(texture_array[atlas_index], default_sampler, tex_coords.xy, layer_index, tex_coords.z);
#else
    if (atlas_index == 1u) {
        return textureSampleLevel(texture_2, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 2u) {
        return textureSampleLevel(texture_3, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 3u) {
        return textureSampleLevel(texture_4, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 4u) {
        return textureSampleLevel(texture_5, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 5u) {
        return textureSampleLevel(texture_6, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 6u) {
        return textureSampleLevel(texture_7, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 7u) {
        return textureSampleLevel(texture_8, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 8u) {
        return textureSampleLevel(texture_9, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 9u) {
        return textureSampleLevel(texture_10, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 10u) {
        return textureSampleLevel(texture_11, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 11u) {
        return textureSampleLevel(texture_12, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 12u) {
        return textureSampleLevel(texture_13, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 13u) {
        return textureSampleLevel(texture_14, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 14u) {
        return textureSampleLevel(texture_15, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    } else if (atlas_index == 15u) {
        return textureSampleLevel(texture_16, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
    }
    return textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
#endif
}



@vertex
fn vs_main(
    v_in: DrawVertex,
    i_in: DrawInstance,
) -> VertexOutput {

    let ui_scale = 2.;

    var vertex_out: VertexOutput;
    vertex_out.clip_position = vec4<f32>(2. * positions_and_colors.data[v_in.index].x * ui_scale / constant_data.screen_width - 1., 1. - 2. * positions_and_colors.data[v_in.index].y * ui_scale / constant_data.screen_height, 0., 1.);
    let color = rgba_from_integer(u32(positions_and_colors.data[v_in.index].w));
    let linear_color = vec4<f32>(linear_from_srgb(color.rgb), color.a / 255.);
    let material_index = u32(meshes.data[i_in.mesh_index].material_index);
    let texture_color = get_texture_color(material_index, TEXTURE_TYPE_BASE_COLOR, u32(v_in.uv_0));
    vertex_out.color = linear_color * texture_color;
    return vertex_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    return v_in.color;
}