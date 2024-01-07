#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read_write> radiance_data_buffer: RadianceDataBuffer;
@group(0) @binding(2)
var finalize_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(3)
var visibility_texture: texture_2d<f32>;
@group(0) @binding(4)
var radiance_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(5)
var depth_texture: texture_depth_2d;

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let dimensions = textureDimensions(finalize_texture);

    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
            
    let visibility_value = textureLoad(visibility_texture, pixel, 0);
    var index = pack4x8unorm(visibility_value);
    
    var radiance = vec4<f32>(0.,0.,0.,1.);
    if(index != 0u) {
        index -= 1u;
        radiance = vec4<f32>(radiance_data_buffer.data[index].radiance, 1.);
    }
    if(constant_data.frame_index > 0u) {
        let prev_value = textureLoad(radiance_texture, pixel);
        let weight = 1. / f32(constant_data.frame_index + 1u);
        radiance = mix(prev_value, radiance, weight);
    }
    textureStore(radiance_texture, pixel, radiance);     
     
    var out_color = vec4<f32>(radiance.rgb, 1.);   
    //out_color = vec4<f32>(tonemap_ACES_Hill(out_color.rgb), 1.);
    //out_color = vec4<f32>(pow(out_color.rgb, vec3<f32>(INV_GAMMA)), 1.); 
    textureStore(finalize_texture, pixel, out_color);
}