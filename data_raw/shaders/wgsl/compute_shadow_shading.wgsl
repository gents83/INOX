#import "common.inc"
#import "utils.inc"
#import "ray_data.inc"
#import "ray_types.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

// Materials and Lights
@group(1) @binding(0)
var<uniform> materials: Materials;
@group(1) @binding(1)
var<uniform> textures: Textures;
@group(1) @binding(2)
var<uniform> lights: Lights;

// Input - Shadow intersections
@group(2) @binding(0)
var<storage, read> shadow_rays: Rays;
@group(2) @binding(1)
var<storage, read> shadow_intersections: Intersections;

// Output - Shadow texture
@group(3) @binding(0)
var shadow_texture: texture_storage_2d<r16float, write>;

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
    
    if (pixel_index >= arrayLength(&shadow_rays.data)) {
        return;
    }

    let ray = shadow_rays.data[pixel_index];
    let intersection = shadow_intersections.data[pixel_index];
    
    var shadow_factor = 1.0;
    
    // Check if ray was active
    if ((ray.flags & RAY_FLAG_ACTIVE) != 0u && (ray.flags & RAY_FLAG_TERMINATED) == 0u) {
        // Check if occluded (instance_id >= 0 means hit)
        if (intersection.instance_id >= 0) {
            shadow_factor = 0.0; // Fully shadowed
        }
    }
    
    // Write shadow factor (1.0 = lit, 0.0 = shadowed)
    textureStore(shadow_texture, pixel, vec4<f32>(shadow_factor, 0.0, 0.0, 0.0));
}
