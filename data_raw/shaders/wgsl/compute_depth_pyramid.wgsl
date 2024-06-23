#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1) 
var input_depth_texture: texture_depth_2d;
@group(0) @binding(2) 
var input_mip_texture : texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) 
var output_mip_texture : texture_storage_2d<r32float, read_write>;
@group(0) @binding(4)
var<uniform> mip_level: u32;


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
    var depth = 0.;
    if(mip_level == 0) {
        depth = textureLoad(input_depth_texture, uv, 0);
    } else {
        let coords = vec2<i32>(p * 2.);
        var prev_depth = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        prev_depth.x = textureLoad(input_mip_texture, coords).x;
        prev_depth.y = textureLoad(input_mip_texture, coords + vec2<i32>(0, 1)).x;
        prev_depth.z = textureLoad(input_mip_texture, coords + vec2<i32>(1, 0)).x;
        prev_depth.w = textureLoad(input_mip_texture, coords + vec2<i32>(1, 1)).x;
        depth = min(min(prev_depth.x, prev_depth.y), min(prev_depth.z, prev_depth.w));
    }
    textureStore(output_mip_texture, uv, vec4<f32>(depth, 0.0, 0.0, 0.0));
}