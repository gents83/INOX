#import "common.inc"
#import "utils.inc"
#import "matrix_utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) instance_id: u32,
};

struct FragmentOutput {
    @location(0) visibility_id: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> transforms: Transforms;

@vertex
fn vs_main(
    @builtin(instance_index) instance_id: u32,
    @location(0) vertex_position: u32,
    instance: Instance,
) -> VertexOutput {
    
    let transform = transforms.data[instance.transform_id];
    let min = transform.bb_min_scale_y.xyz;
    let size = abs(transform.bb_max_scale_z.xyz - min);
    let p = min + unpack_unorm_to_3_f32(vertex_position) * size;
    let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w);
    let v = transform_vector(p, transform.position_scale_x.xyz, transform.orientation, scale);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = constant_data.view_proj * vec4<f32>(v, 1.);
    vertex_out.instance_id = instance_id + 1u;    

    return vertex_out;
}

@fragment
fn fs_main(
    @builtin(primitive_index) primitive_index: u32,
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;
    fragment_out.visibility_id = (v_in.instance_id << 8u) | primitive_index;   
    return fragment_out;
}