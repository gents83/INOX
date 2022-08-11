#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec3<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};


struct BlitPassData {
    source_texture_index: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
};

@group(0) @binding(0)
var<uniform> data: BlitPassData;
@group(0) @binding(1)
var<storage, read> textures: Textures;



@group(1) @binding(0)
var default_sampler: sampler;
@group(1) @binding(1)
var unfiltered_sampler: sampler;
@group(1) @binding(2)
var depth_sampler: sampler_comparison;

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
@group(1) @binding(3)
var texture_array: binding_array<texture_2d_array<f32>, 16>; //MAX_TEXTURE_ATLAS_COUNT
#else
@group(1) @binding(3)
var texture_1: texture_2d_array<f32>;
@group(1) @binding(4)
var texture_2: texture_2d_array<f32>;
@group(1) @binding(5)
var texture_3: texture_2d_array<f32>;
@group(1) @binding(6)
var texture_4: texture_2d_array<f32>;
@group(1) @binding(7)
var texture_5: texture_2d_array<f32>;
@group(1) @binding(8)
var texture_6: texture_2d_array<f32>;
@group(1) @binding(9)
var texture_7: texture_2d_array<f32>;
@group(1) @binding(10)
var texture_8: texture_2d_array<f32>;;
#endif


fn get_texture_color(tex_coords_and_texture_index: vec3<f32>) -> vec4<f32> {
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
    }
    return textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, tex_coords.z);
#endif
}


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    //only one triangle, exceeding the viewport size
	let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
	let pos = vec4<f32>(uv * vec2<f32>(2., -2.) + vec2<f32>(-1., 1.), 0., 1.);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = pos;
    vertex_out.uv = vec3<f32>(uv.xy, f32(data.source_texture_index));
    return vertex_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = get_texture_color(v_in.uv);
    return texture_color;
}