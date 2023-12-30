#import "common.inc"
#import "utils.inc"

struct DebugVertex {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

#import "matrix_utils.inc"

@vertex
fn vs_main(
    v_in: DebugVertex,
) -> VertexOutput {

    var vertex_out: VertexOutput;
    vertex_out.clip_position = constant_data.proj * constant_data.view * vec4<f32>(v_in.position, 1.);
    vertex_out.color = unpack_unorm_to_4_f32(v_in.color);

    return vertex_out;
}

@fragment
fn fs_main(
    @builtin(primitive_index) primitive_index: u32,
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;
    fragment_out.color = v_in.color;
    return fragment_out;
}