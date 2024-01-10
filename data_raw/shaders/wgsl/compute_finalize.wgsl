#import "common.inc"
#import "utils.inc"
#import "color_utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read_write> radiance_data_buffer: RadianceDataBuffer;
@group(0) @binding(2)
var finalize_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(3)
var binding_texture: texture_2d<f32>;
@group(0) @binding(4)
var radiance_texture: texture_storage_2d<rgba8unorm, read_write>;

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
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    }     
            
    let binding_dimensions = textureDimensions(binding_texture);
    let binding_scale = vec2<f32>(binding_dimensions) / vec2<f32>(dimensions);
    let binding_pixel = vec2<u32>((vec2<f32>(0.5) + vec2<f32>(pixel)) * binding_scale);
    let binding_value = textureLoad(binding_texture, binding_pixel, 0);
    var index = pack4x8unorm(binding_value);
    
    var radiance = vec4<f32>(0.,0.,0.,1.);
    if(index != 0u) {
        index -= 1u;
        radiance = vec4<f32>(radiance_data_buffer.data[index].radiance, 1.);
    }
    let radiance_dimensions = textureDimensions(radiance_texture);
    let radiance_scale = vec2<f32>(radiance_dimensions) / vec2<f32>(dimensions);
    let radiance_pixel = vec2<u32>((vec2<f32>(0.5) + vec2<f32>(pixel)) * radiance_scale);
    if(constant_data.frame_index > 0u) {
        let prev_value = textureLoad(radiance_texture, radiance_pixel);
        let frame_index = f32(constant_data.frame_index + 1u);
        let weight = 1. / frame_index;
        radiance = mix(prev_value, radiance, weight);
    }
    textureStore(radiance_texture, radiance_pixel, radiance);     
     
    var out_color = vec4<f32>(radiance.rgb, 1.);   
    //out_color = vec4<f32>(tonemap_ACES_Hill(out_color.rgb), 1.);
    //out_color = vec4<f32>(linearTosRGB(out_color.rgb), 1.); 
    textureStore(finalize_texture, pixel, out_color);
}