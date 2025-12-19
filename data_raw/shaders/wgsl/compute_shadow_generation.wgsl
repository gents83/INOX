#import "common.inc"
#import "utils.inc"
#import "vertex_utils.inc"
#import "visibility_utils.inc"
#import "ray_data.inc"
#import "ray_types.inc"
#import "sampling.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var visibility_texture: texture_2d<u32>;
@group(0) @binding(2)
var depth_texture: texture_depth_2d;

// Geometry
@group(1) @binding(0)
var<storage, read> indices: Indices;
@group(1) @binding(1)
var<storage, read> vertices_positions: VerticesPositions;
@group(1) @binding(2)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(1) @binding(3)
var<storage, read> instances: Instances;
@group(1) @binding(4)
var<storage, read> transforms: Transforms;
@group(1) @binding(5)
var<storage, read> meshes: Meshes;
@group(1) @binding(6)
var<storage, read> meshlets: Meshlets;

// Materials and Lights
@group(2) @binding(0)
var<uniform> materials: Materials;
@group(2) @binding(1)
var<uniform> textures: Textures;
@group(2) @binding(2)
var<uniform> lights: Lights;

// Output - Shadow rays
@group(3) @binding(0)
var<storage, read_write> shadow_rays: Rays;

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

    let visibility_dimensions = textureDimensions(visibility_texture);
    let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
    let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
    let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
    let visibility_id = visibility_value.r;

    // Check if we have valid visibility
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        // No geometry hit - mark shadow ray as terminated
        if (pixel_index < arrayLength(&shadow_rays.data)) {
            shadow_rays.data[pixel_index].t_max = -1.0;
            shadow_rays.data[pixel_index].flags = RAY_FLAG_TERMINATED;
        }
        return;
    }

    let depth_dimensions = textureDimensions(depth_texture);
    let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
    let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
    let depth = textureLoad(depth_texture, depth_pixel, 0);
    
    let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth);
    
    // Reconstruct pixel data from visibility
    var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
    
    // For simplicity, generate one shadow ray per pixel towards first light
    // In a more sophisticated implementation, we'd generate multiple rays for multiple lights
    let num_lights = constant_data.num_lights;
    
    if (num_lights > 0u && pixel_index < arrayLength(&shadow_rays.data)) {
        // Use first light for now
        let light_index = hash(constant_data.frame_index) % num_lights;
        let light = lights.data[light_index];
        
        var light_dir: vec3<f32>;
        var light_distance: f32 = MAX_TRACING_DISTANCE;
        
        if ((light.light_type & LIGHT_TYPE_DIRECTIONAL) != 0u) {
            // Directional light
            light_dir = normalize(-light.direction);
            light_distance = MAX_TRACING_DISTANCE;
        } else if ((light.light_type & LIGHT_TYPE_POINT) != 0u) {
            // Point light
            let to_light = light.position - pixel_data.world_pos;
            light_distance = length(to_light);
            light_dir = normalize(to_light);
        } else if ((light.light_type & LIGHT_TYPE_SPOT) != 0u) {
            // Spot light
            let to_light = light.position - pixel_data.world_pos;
            light_distance = length(to_light);
            light_dir = normalize(to_light);
        } else {
            // Invalid light type - terminate ray
            shadow_rays.data[pixel_index].t_max = -1.0;
            shadow_rays.data[pixel_index].flags = RAY_FLAG_TERMINATED;
            return;
        }
        
        // Offset origin slightly along normal to avoid self-intersection
        let ray_origin = pixel_data.world_pos + pixel_data.normal * 0.001;
        
        // Create shadow ray
        shadow_rays.data[pixel_index].origin = ray_origin;
        shadow_rays.data[pixel_index].direction = light_dir;
        shadow_rays.data[pixel_index].t_min = 0.001;
        shadow_rays.data[pixel_index].t_max = light_distance;
        shadow_rays.data[pixel_index].throughput = vec3<f32>(1.0);
        shadow_rays.data[pixel_index].pixel_index = pixel_index;
        shadow_rays.data[pixel_index].ray_type = RAY_TYPE_SHADOW;
        shadow_rays.data[pixel_index].bounce_count = 0u;
        shadow_rays.data[pixel_index].flags = RAY_FLAG_ACTIVE | RAY_FLAG_ANY_HIT;
        shadow_rays.data[pixel_index]._padding = 0u;
    }
}
