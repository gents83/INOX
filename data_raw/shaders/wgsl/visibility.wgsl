#import "common.inc"
#import "utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) id: u32,
};

struct FragmentOutput {
    @location(0) output: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> positions: Positions;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(4)
var<storage, read> meshes_aabb: AABBs;

@vertex
fn vs_main(
    @builtin(instance_index) meshlet_id: u32,
    v_in: Vertex,
) -> VertexOutput {
    
    let meshlet = &meshlets.data[meshlet_id];
    let mesh_id = (*meshlet).mesh_index;
    let mesh = &meshes.data[mesh_id];
    let aabb = &meshes_aabb.data[mesh_id];

    let mvp = constant_data.proj * constant_data.view;
    
    let aabb_size = abs((*aabb).max - (*aabb).min);
    let p = (*aabb).min + decode_as_vec3(positions.data[v_in.position_and_color_offset]) * aabb_size;
    let world_position = vec4<f32>(transform_vector(p, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.0);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * world_position;
    vertex_out.id = meshlet_id + 1u;    

    return vertex_out;
}

@fragment
fn fs_main(
    @builtin(primitive_index) primitive_index: u32,
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;
    let visibility_id = v_in.id << 8u | primitive_index;   
    fragment_out.output = unpack4x8unorm(visibility_id);    
    return fragment_out;
}