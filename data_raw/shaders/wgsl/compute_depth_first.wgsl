#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1) 
var input_depth_texture: texture_depth_2d;
@group(0) @binding(2) 
var output_mip_texture : texture_storage_2d<r32float, read_write>;


@compute
@workgroup_size(8, 8, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    var p = vec2<f32>(global_invocation_id.xy);
    let uv = vec2<i32>(p);
    let depth = textureLoad(input_depth_texture, uv, 0);
    textureStore(output_mip_texture, uv, vec4<f32>(depth, 0.0, 0.0, 0.0));
}