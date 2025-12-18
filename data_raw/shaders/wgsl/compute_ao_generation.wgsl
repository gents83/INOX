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

// Output - AO rays
@group(2) @binding(0)
var<storage, read_write> ao_rays: Rays;

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
        // No geometry hit - mark AO ray as terminated
        if (pixel_index < arrayLength(&ao_rays.data)) {
            ao_rays.data[pixel_index].t_max = -1.0;
            ao_rays.data[pixel_index].flags = RAY_FLAG_TERMINATED;
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
    
    if (pixel_index < arrayLength(&ao_rays.data)) {
        // Generate AO ray with cosine-weighted hemisphere sampling
        var seed = vec2<u32>(pixel_index, constant_data.frame_index);
        let rnd = get_random_numbers(&seed);
        
        let N = normalize(pixel_data.normal);
        let up = select(vec3(1., 0., 0.), vec3(0., 1., 0.), abs(N.z) < 0.999);
        let tangent = normalize(cross(up, N));
        let bitangent = cross(N, tangent);
        let tbn = mat3x3<f32>(tangent, bitangent, N);
        
        // Cosine-weighted hemisphere sampling for AO
        let local_dir = sample_cosine_weighted_hemisphere(rnd);
        let ao_direction = normalize(tbn * local_dir);
        
        // Offset origin slightly along normal
        let ray_origin = pixel_data.world_pos + N * 0.001;
        
        // Create AO ray with limited distance
        ao_rays.data[pixel_index].origin = ray_origin;
        ao_rays.data[pixel_index].direction = ao_direction;
        ao_rays.data[pixel_index].t_min = 0.001;
        ao_rays.data[pixel_index].t_max = AO_MAX_DISTANCE;
        ao_rays.data[pixel_index].throughput = vec3<f32>(1.0);
        ao_rays.data[pixel_index].pixel_index = pixel_index;
        ao_rays.data[pixel_index].ray_type = RAY_TYPE_AO;
        ao_rays.data[pixel_index].bounce_count = 0u;
        ao_rays.data[pixel_index].flags = RAY_FLAG_ACTIVE | RAY_FLAG_ANY_HIT;
        ao_rays.data[pixel_index]._padding = 0u;
    }
}
