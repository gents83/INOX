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
var<storage, read> matrices: Matrices;

@vertex
fn vs_main(
    v_in: DrawVertex,
    i_in: DrawInstance,
) -> VertexOutput {
    let instance_matrix = matrices.data[i_in.matrix_index];
    let position = positions_and_colors.data[v_in.position_and_color_offset].xyz;
    let color = u32(positions_and_colors.data[v_in.position_and_color_offset].w);

    let mvp = constant_data.proj * constant_data.view * instance_matrix;
    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * vec4<f32>(position, 1.0);
    vertex_out.color = unpack_unorm_to_4_f32(color);

    return vertex_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    return v_in.color;
}