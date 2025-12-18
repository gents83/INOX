#import "common.inc"
#import "utils.inc"
#import "ray_data.inc"
#import "ray_types.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

// Input - AO intersections
@group(1) @binding(0)
var<storage, read> ao_rays: Rays;
@group(1) @binding(1)
var<storage, read> ao_intersections: Intersections;

// Output - AO texture
@group(2) @binding(0)
var ao_texture: texture_storage_2d<r16float, write>;

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    }

    let pixel_index = pixel.y * dimensions.x + pixel.x;
    
    if (pixel_index >= arrayLength(&ao_rays.data)) {
        return;
    }

    let ray = ao_rays.data[pixel_index];
    let intersection = ao_intersections.data[pixel_index];
    
    var ao_factor = 1.0;
    
    // Check if ray was active
    if ((ray.flags & RAY_FLAG_ACTIVE) != 0u && (ray.flags & RAY_FLAG_TERMINATED) == 0u) {
        // Check if occluded (instance_id >= 0 means hit)
        if (intersection.instance_id >= 0) {
            // AO based on distance - closer occlusion = darker
            let hit_distance = intersection.t;
            if (hit_distance > 0.0 && hit_distance < AO_MAX_DISTANCE) {
                // Simple distance-based AO falloff
                ao_factor = hit_distance / AO_MAX_DISTANCE;
                ao_factor = ao_factor * ao_factor; // Square for smoother falloff
            }
        }
    }
    
    // Write AO factor (1.0 = no occlusion, 0.0 = fully occluded)
    textureStore(ao_texture, pixel, vec4<f32>(ao_factor, 0.0, 0.0, 0.0));
}
