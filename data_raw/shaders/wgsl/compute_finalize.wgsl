#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var render_target: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(2)
var visibility_texture: texture_2d<f32>;
@group(0) @binding(3)
var gbuffer_texture: texture_2d<f32>;
@group(0) @binding(4)
var radiance_texture: texture_2d<f32>;
@group(0) @binding(5)
var depth_texture: texture_depth_2d;

const WORKGROUP_SIZE: u32 = 4u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let source_dimensions = textureDimensions(radiance_texture);
    let target_dimensions = textureDimensions(render_target);
    let scale = vec2<f32>(source_dimensions) / vec2<f32>(target_dimensions);

    let target_pixel = vec2<u32>(workgroup_id.x * WORKGROUP_SIZE + local_invocation_id.x, 
                                 workgroup_id.y * WORKGROUP_SIZE + local_invocation_id.y);
    if (target_pixel.x > target_dimensions.x || target_pixel.y > target_dimensions.y) {
        return;
    }    
    let source_pixel = vec2<u32>(vec2<f32>(target_pixel) * scale);
    let radiance = textureLoad(radiance_texture, source_pixel, 0);  
    let gbuffer = textureLoad(gbuffer_texture, source_pixel, 0);        
    var out_color = vec4<f32>(gbuffer.rgb + radiance.rgb, 1.);    
    //out_color = vec4<f32>(tonemap_ACES_Hill(out_color.rgb), 1.);
    //out_color = vec4<f32>(pow(out_color.rgb, vec3<f32>(INV_GAMMA)), 1.);
    
    if(constant_data.frame_index > 0u) {
        var prev_value = textureLoad(render_target, target_pixel);
        let weight = 1. / f32(constant_data.frame_index + 1u);
        out_color = mix(prev_value, out_color, weight);
    }
    
    textureStore(render_target, target_pixel, out_color);
}