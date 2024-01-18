#import "common.inc"
#import "utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) id: u32,
};

struct FragmentOutput {
    @location(0) output: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

@vertex
fn vs_main(
    @builtin(instance_index) meshlet_id: u32,
    v_in: RuntimeVertexData,
) -> VertexOutput {
    var vertex_out: VertexOutput;
    vertex_out.clip_position = constant_data.view_proj * vec4<f32>(v_in.world_pos, 1.);
    vertex_out.id = meshlet_id + 1u;    

    return vertex_out;
}

@fragment
fn fs_main(
    @builtin(primitive_index) primitive_index: u32,
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;
    let visibility_id = (v_in.id << 8u) | primitive_index;   
    fragment_out.output = visibility_id;    
    return fragment_out;
}