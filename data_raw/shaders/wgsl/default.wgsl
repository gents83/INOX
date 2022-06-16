#import "utils.wgsl"
#import "common.wgsl"

struct VertexInput {
    @builtin(vertex_index) index: u32,
};

struct InstanceInput {
    @builtin(instance_index) index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

@group(0) @binding(1)
var<storage, read> instances: Instances;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> vertices: Vertices;
@group(0) @binding(4)
var<storage, read> positions: Positions;
@group(0) @binding(5)
var<storage, read> colors: Colors;
@group(0) @binding(6)
var<storage, read> matrices: Matrices;

@vertex
fn vs_main(
    v_in: VertexInput,
    i_in: InstanceInput,
) -> VertexOutput {

    let mesh_index = instances.data[i_in.index].mesh_index;
    let matrix_index = meshes.data[mesh_index].matrix_index;
    let vertex_data = vertices.data[v_in.index];

    let instance_matrix = matrices.data[matrix_index];
    let position = positions.data[vertex_data.position_offset];
    let color = colors.data[vertex_data.color_offset];

    let world_position = instance_matrix * vec4<f32>(position, 1.0);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = constant_data.proj * constant_data.view * world_position;
    vertex_out.color = rgba_from_integer(color);

    return vertex_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    return v_in.color;
}