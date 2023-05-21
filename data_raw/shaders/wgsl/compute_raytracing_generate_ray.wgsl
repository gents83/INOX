#import "common.inc"

struct Data {
    width: u32,
    height: u32,
    _padding: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> data: Data;

@group(1) @binding(0)
var<storage, read_write> rays: Rays;

fn unproject(ncd_pos: vec2<f32>, depth: f32) -> vec3<f32> 
{    
    var world_pos = constant_data.inverse_view_proj * vec4<f32>(ncd_pos, depth, 1. );
    world_pos /= world_pos.w;
    return world_pos.xyz;
}

fn compute_ray(image_pixel: vec2<u32>, image_size: vec2<u32>) -> Ray {
    var clip_coords = 2. * (vec2<f32>(image_pixel) / vec2<f32>(image_size)) - vec2<f32>(1., 1.);
    clip_coords.y = -clip_coords.y;
    
    let origin = unproject(clip_coords.xy, -1.);
    let far = unproject(clip_coords.xy, 1.);
    let direction = normalize(far - origin);
    
    return Ray(origin, 0., direction, MAX_FLOAT);
}


@compute
@workgroup_size(8, 8, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {  
    let pixel = vec2<u32>(workgroup_id.x * 8u + local_invocation_id.x, 
                          workgroup_id.y * 8u + local_invocation_id.y);
    if (pixel.x >= data.width || pixel.y >= data.height) {
        return;
    }    
    // Create a ray with the current fragment as the origin.
    let index = pixel.y * data.width + pixel.x;
    rays.data[index] = compute_ray(pixel, vec2<u32>(data.width, data.height));
}