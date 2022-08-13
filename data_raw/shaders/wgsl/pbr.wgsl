#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec3<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};


struct PBRPassData {
    gbuffer1: u32,
    gbuffer2: u32,
    gbuffer3: u32,
    depth: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> data: PBRPassData;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(4)
var<storage, read> materials: Materials;
@group(0) @binding(5)
var<storage, read> textures: Textures;
@group(0) @binding(6)
var<storage, read> lights: Lights;

#import "texture_utils.wgsl"
#import "material_utils.wgsl"
#import "pbr_utils.wgsl"

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    //only one triangle, exceeding the viewport size
	let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
	let pos = vec4<f32>(uv * vec2<f32>(2., -2.) + vec2<f32>(-1., 1.), 0., 1.);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = pos;
    vertex_out.uv = vec3<f32>(uv.xy, f32(in_vertex_index));
    return vertex_out;
}


@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {        
    let vertex_color = sample_texture(vec3<f32>(v_in.uv.xy, f32(data.gbuffer1)));
    let meshlet_id = pack4x8unorm(sample_texture(vec3<f32>(v_in.uv.xy, f32(data.gbuffer3))));
    if meshlet_id == 0u {
        return vec4<f32>(0., 0., 0., 0.);
    }

    var color = vec4<f32>(0., 0., 0., 0.);
    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let meshlet_color = hash(meshlet_id - 1u);
        color = vec4<f32>(vec3<f32>(
            f32(meshlet_color & 255u), 
            f32((meshlet_color >> 8u) & 255u), 
            f32((meshlet_color >> 16u) & 255u)) / 255., 
            1.
        );
    } else {
        let uv_0 = pack4x8unorm(sample_texture(vec3<f32>(v_in.uv.xy, f32(data.gbuffer3 + 1u))));
        let uv_1 = pack4x8unorm(sample_texture(vec3<f32>(v_in.uv.xy, f32(data.gbuffer3 + 2u))));
        let uv_2 = pack4x8unorm(sample_texture(vec3<f32>(v_in.uv.xy, f32(data.gbuffer3 + 3u))));
        let uv_3 = pack4x8unorm(sample_texture(vec3<f32>(v_in.uv.xy, f32(data.gbuffer3 + 4u))));
        let uv_set = vec4<u32>(uv_0, uv_1, uv_2, uv_3);

        let mesh_id = meshlets.data[meshlet_id - 1u].mesh_index;
        let material_id = u32(meshes.data[mesh_id].material_index);
        let texture_color = sample_material_texture(material_id, TEXTURE_TYPE_BASE_COLOR, uv_set);

        let alpha = compute_alpha(material_id, vertex_color.a);
        if alpha < 0. {
            discard;
        }

        color = vec4<f32>(vertex_color.rgb * texture_color.rgb, alpha);

        let packed_normal = unpack2x16float(pack4x8unorm(sample_texture(vec3<f32>(v_in.uv.xy, f32(data.gbuffer2)))));
        let n = unpack_normal(packed_normal);        
        let world_pos = compute_world_position_from_depth(v_in.uv.xy, data.depth);
        color = compute_brdf(world_pos, n, material_id, color, uv_set);
    }

    return color;
}