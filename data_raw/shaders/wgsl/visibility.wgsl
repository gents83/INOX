#import "common.wgsl"

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
var<storage, read> positions_and_colors: PositionsAndColors;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(4)
var<storage, read> matrices: Matrices;

@vertex
fn vs_main(
    @builtin(instance_index) instance_index: u32,
    v_in: DrawVertex,
    i_in: DrawInstance,
) -> VertexOutput {
    
    let meshlet_id = extractBits(instance_index, 0u, 24u);
    let meshlet = &meshlets.data[meshlet_id];
    let mesh_id = (*meshlet).mesh_index;
    let mesh = &meshes.data[mesh_id];
    let matrix_id = (*mesh).matrix_index;

    let instance_matrix = &matrices.data[matrix_id];
    let p = &positions_and_colors.data[v_in.position_and_color_offset];
    let world_position = (*instance_matrix) * vec4<f32>((*p).xyz, 1.0);
    let mvp = constant_data.proj * constant_data.view;

    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * world_position;
    vertex_out.id = instance_index;    
    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;
    fragment_out.output = unpack4x8unorm(v_in.id);    
    return fragment_out;
}