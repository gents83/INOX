#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) params: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> positions_and_colors: PositionsAndColors;
@group(0) @binding(2)
var<storage, read> uvs: UVs;

@group(1) @binding(0)
var<storage, read> matrices: Matrices;
@group(1) @binding(1)
var<storage, read> meshes: Meshes;
@group(1) @binding(2)
var<storage, read> meshlets: Meshlets;
@group(1) @binding(3)
var<storage, read> materials: Materials;
@group(1) @binding(4)
var<storage, read> textures: Textures;




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


fn get_texture_color(tex_coords_and_texture_index: vec3<f32>) -> vec4<f32> {
    let texture_data_index = i32(tex_coords_and_texture_index.z);
    var tex_coords = vec3<f32>(0.0, 0.0, 0.0);
    if (texture_data_index < 0) {
        return vec4<f32>(tex_coords, 0.);
    }
    let atlas_index = textures.data[texture_data_index].texture_index;
    let layer_index = i32(textures.data[texture_data_index].layer_index);

    tex_coords.x = (textures.data[texture_data_index].area.x + tex_coords_and_texture_index.x * textures.data[texture_data_index].area.z) / textures.data[texture_data_index].total_width;
    tex_coords.y = (textures.data[texture_data_index].area.y + tex_coords_and_texture_index.y * textures.data[texture_data_index].area.w) / textures.data[texture_data_index].total_height;
    tex_coords.z = f32(layer_index);

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
    let instance_matrix = matrices.data[i_in.matrix_index];
    let position = positions_and_colors.data[v_in.position_and_color_offset].xyz;

    let mvp = constant_data.proj * constant_data.view * instance_matrix;
    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * vec4<f32>(position, 1.0);

    let instance_id = i_in.index;
    let mesh_id = i_in.mesh_index;
    var i = meshes.data[mesh_id].meshlet_offset + meshes.data[mesh_id].meshlet_count - 1u;
    var meshlet_id = f32(i + 1u);
    while(i > 0u) {
        if ((v_in.index - meshes.data[mesh_id].vertex_offset) > meshlets.data[i].vertex_offset) {
            break;
        }
        meshlet_id = f32(i - 1u);
        i -= 1u;
    }

    let color = positions_and_colors.data[v_in.position_and_color_offset].w;
    vertex_out.params = vec4<f32>(f32(instance_id), f32(mesh_id), f32(meshlet_id), color);

    let material_id = meshes.data[mesh_id].material_index;
    let texture_id = materials.data[material_id].textures_indices[TEXTURE_TYPE_BASE_COLOR];
    var uv = vec2<f32>(0., 0.);
    if (materials.data[material_id].textures_coord_set[TEXTURE_TYPE_BASE_COLOR] == 0u) {
        uv = uvs.data[v_in.uvs_offset.x].xy;
    } else if (materials.data[material_id].textures_coord_set[TEXTURE_TYPE_BASE_COLOR] == 1u) {
        uv = uvs.data[v_in.uvs_offset.y].xy;
    } else if (materials.data[material_id].textures_coord_set[TEXTURE_TYPE_BASE_COLOR] == 2u) {
        uv = uvs.data[v_in.uvs_offset.z].xy;
    } else if (materials.data[material_id].textures_coord_set[TEXTURE_TYPE_BASE_COLOR] == 3u) {
        uv = uvs.data[v_in.uvs_offset.w].xy;
    }
    let texture_coords = vec3<f32>(uv, f32(texture_id));
    

    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> @location(0) vec4<f32> {
    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let h = hash(u32(v_in.params.b));
        return vec4<f32>(vec3<f32>(f32(h & 255u), f32((h >> 8u) & 255u), f32((h >> 16u) & 255u)) / 255., 1.);
    }
    return v_in.params;
}