#import "common.inc"
#import "utils.inc"
#import "matrix_utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) instance_id: u32,
    @location(1) @interpolate(flat) meshlet_id: u32,
};

struct FragmentOutput {
    @location(0) visibility_id: u32,
    @location(1) instance_id: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

@vertex
fn vs_main(
    @builtin(instance_index) instance_id: u32,
    @location(0) vertex_position: u32,
    instance: Instance,
) -> VertexOutput {
    
    let bb_min = transform_vector(instance.mesh_local_bb_min, instance.position, instance.orientation, instance.scale);
    let bb_max = transform_vector(instance.mesh_local_bb_max, instance.position, instance.orientation, instance.scale);
    let size = bb_max - bb_min;
    let p = bb_min + unpack_unorm_to_3_f32(vertex_position) * size;

    var vertex_out: VertexOutput;
    vertex_out.clip_position = constant_data.view_proj * vec4<f32>(p, 1.);
    vertex_out.instance_id = instance_id + 1u;    
    vertex_out.meshlet_id = instance.meshlet_index + 1u;    

    return vertex_out;
}

@fragment
fn fs_main(
    @builtin(primitive_index) primitive_index: u32,
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;
    fragment_out.visibility_id = (v_in.meshlet_id << 8u) | primitive_index;   
    fragment_out.instance_id = v_in.instance_id;    
    return fragment_out;
}