#import "common.inc"
#import "utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) instance_id: u32,
    @location(1) @interpolate(flat) meshlet_id: u32,
};

struct Instance {
    @location(1) mesh_bb_min: vec3<f32>,
    @location(2) mesh_index: u32,
    @location(3) mesh_bb_max: vec3<f32>,
    @location(4) meshlet_index: u32,
}

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
    
    let size =  instance.mesh_bb_max - instance.mesh_bb_min;
    let p = instance.mesh_bb_min + unpack_unorm_to_3_f32(vertex_position) * size;

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