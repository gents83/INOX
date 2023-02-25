#import "common.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

@group(1) @binding(0)
var<storage, read_write> rays: Rays;

@group(2) @binding(0)
var render_target: texture_storage_2d<rgba8unorm, read_write>;

#import "matrix_utils.inc"

fn compute_ray(image_pixel: vec2<u32>, image_size: vec2<u32>) -> Ray {
    var clip_coords = 2. * (vec2<f32>(image_pixel) / vec2<f32>(image_size)) - vec2<f32>(1., 1.);
    clip_coords.y = -clip_coords.y;
    
    let origin = unproject(clip_coords.xy, -1.);
    let far = unproject(clip_coords.xy, 1.);
    let direction = normalize(far - origin);
    
    return Ray(origin, 0., direction, MAX_FLOAT);
}


@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = vec2<u32>(textureDimensions(render_target));    
    let pixel = vec2<u32>(workgroup_id.x * 16u + local_invocation_id.x, 
                          workgroup_id.y * 16u + local_invocation_id.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y)
    {
        return;
    }    
    // Create a ray with the current fragment as the origin.
    let index = pixel.y * dimensions.x + pixel.x;
    rays.data[index] = compute_ray(pixel, dimensions);
}