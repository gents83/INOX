#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> radiance_data_buffer: RadianceDataBuffer;
@group(0) @binding(2)
var finalize_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(3)
var visibility_texture: texture_2d<f32>;
@group(0) @binding(4)
var gbuffer_texture: texture_2d<f32>;
@group(0) @binding(5)
var radiance_texture: texture_storage_2d<rgba32float, read_write>;
@group(0) @binding(6)
var depth_texture: texture_depth_2d;

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = textureDimensions(finalize_texture);

    let pixel = vec2<u32>(workgroup_id.x * WORKGROUP_SIZE + local_invocation_id.x, 
                          workgroup_id.y * WORKGROUP_SIZE + local_invocation_id.y);
    if (pixel.x > dimensions.x || pixel.y > dimensions.y) {
        return;
    }    
    
    let radiance_dimensions = textureDimensions(radiance_texture);
    let radiance_scale = vec2<f32>(radiance_dimensions) / vec2<f32>(dimensions);
    let radiance_pixel = vec2<u32>(vec2<f32>(pixel) * radiance_scale);
    
    let index = radiance_pixel.y * radiance_dimensions.x + radiance_pixel.x;    
    var radiance = vec4<f32>(radiance_data_buffer.data[index].radiance, 1.);
    if(constant_data.frame_index > 0u) {
        let prev_value = textureLoad(radiance_texture, radiance_pixel);
        let weight = 1. / f32(constant_data.frame_index + 1u);
        radiance = mix(prev_value, radiance, weight);
    }
    textureStore(radiance_texture, radiance_pixel, radiance);
    
    let gbuffer_dimensions = textureDimensions(gbuffer_texture);
    let gbuffer_scale = vec2<f32>(gbuffer_dimensions) / vec2<f32>(dimensions);
    let gbuffer_pixel = vec2<u32>(vec2<f32>(pixel) * gbuffer_scale); 
    let gbuffer = textureLoad(gbuffer_texture, gbuffer_pixel, 0);      
     
    var out_color = vec4<f32>(gbuffer.rgb + radiance.rgb, 1.);   
    //out_color = vec4<f32>(tonemap_ACES_Hill(out_color.rgb), 1.);
    //out_color = vec4<f32>(pow(out_color.rgb, vec3<f32>(INV_GAMMA)), 1.); 
    textureStore(finalize_texture, pixel, out_color);
}