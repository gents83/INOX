#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var render_target: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(2)
var visibility_texture: texture_2d<f32>;
@group(0) @binding(3)
var radiance_texture: texture_2d<f32>;
@group(0) @binding(4)
var depth_texture: texture_depth_2d;


const MAX_WORKGROUP_SIZE: u32 = 16u*16u;
var<workgroup> jobs_count: atomic<u32>;

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let source_dimensions = textureDimensions(radiance_texture);
    let target_dimensions = textureDimensions(render_target);
    let scale = vec2<f32>(source_dimensions) / vec2<f32>(target_dimensions);
    atomicStore(&jobs_count, MAX_WORKGROUP_SIZE);
    
    var job_index = 0u;
    while(job_index < MAX_WORKGROUP_SIZE)
    {
        let target_pixel = vec2<u32>(workgroup_id.x * 16u + job_index % 16u, 
                              workgroup_id.y * 16u + job_index / 16u);
        if (target_pixel.x >= target_dimensions.x || target_pixel.y >= target_dimensions.y) {
            job_index = MAX_WORKGROUP_SIZE - atomicSub(&jobs_count, 1u);
            continue;
        }    
        let source_pixel = vec2<u32>(vec2<f32>(target_pixel) * scale);
        var out_color = vec4<f32>(0.);
        if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS) != 0) {
            let visibility_output = textureLoad(visibility_texture, source_pixel, 0);
            let visibility_id = pack4x8unorm(visibility_output);
            if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
                let meshlet_id = (visibility_id >> 8u); 
                let meshlet_color = hash(meshlet_id + 1u);
                out_color = vec4<f32>(vec3<f32>(
                    f32(meshlet_color & 255u),
                    f32((meshlet_color >> 8u) & 255u),
                    f32((meshlet_color >> 16u) & 255u)
                ) / 255., 1.);
            }
        } 
        else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_VISIBILITY_BUFFER) != 0) {
            out_color = textureLoad(visibility_texture, source_pixel, 0);
        } 
        else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER) != 0) {
            let depth = textureLoad(depth_texture, source_pixel, 0);
            let v = vec3<f32>(1. - depth);
            out_color = vec4<f32>(v, 1.);
        } 
        else {
            out_color = textureLoad(radiance_texture, source_pixel, 0);            
            //out_color = vec4<f32>(tonemap_ACES_Hill(out_color.rgb), 1.);
            //out_color = vec4<f32>(pow(out_color.rgb, vec3<f32>(INV_GAMMA)), 1.);
            
            if(constant_data.frame_index > 0u) {
                var prev_value = textureLoad(render_target, target_pixel);
                let weight = 1. / f32(constant_data.frame_index + 1u);
                out_color = mix(prev_value, out_color, weight);
            }
        }
         
        textureStore(render_target, target_pixel, out_color);
        job_index = MAX_WORKGROUP_SIZE - atomicSub(&jobs_count, 1u);
    }
}