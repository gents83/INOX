#import "common.inc"
#import "utils.inc"
#import "vertex_utils.inc"
#import "visibility_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "ray_data.inc"
#import "ray_types.inc"
#import "sampling.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var visibility_texture: texture_2d<u32>;
@group(0) @binding(2)
var depth_texture: texture_depth_2d;
@group(0) @binding(3)
var direct_lighting_texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4)
var<storage, read_write> rays: Rays;

// Group 1: Geometry
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

// Group 2: Materials, Textures, Lights
@group(2) @binding(0)
var<uniform> materials: Materials;
@group(2) @binding(1)
var<uniform> textures: Textures;
@group(2) @binding(2)
var<uniform> lights: Lights;

#import "texture_utils.inc"

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

    var direct_light = vec3<f32>(0.0);

    let visibility_dimensions = textureDimensions(visibility_texture);
    let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
    let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
    let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
    let visibility_id = visibility_value.r;

    if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth);
        
        // Reconstruct pixel data from visibility
        var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
        
        // Compute PBR lighting (direct lighting only)
        let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);
        direct_light = material_info.f_color.rgb;
        
        // Generate bounce ray for path tracing
        let ray_index = pixel.y * DEFAULT_WIDTH + pixel.x;
        let N = normalize(pixel_data.normal);
        
        // Simple cosine-weighted hemisphere sampling for diffuse bounce
        var seed = vec2<u32>(ray_index, constant_data.frame_index);
        let random = get_random_numbers(&seed);
        let random_dir = get_random_numbers(&seed);
        
        // Cosine-weighted hemisphere sample
        let local_dir = sample_cosine_weighted_hemisphere(random_dir.xy);
        
        // Transform to world space (TBN matrix)
        let up = select(vec3(1., 0., 0.), vec3(0., 1., 0.), abs(N.z) < 0.999);
        let tangent = normalize(cross(up, N));
        let bitangent = cross(N, tangent);
        let ray_dir = normalize(mat3x3<f32>(tangent, bitangent, N) * local_dir);
        
        // Store bounce ray
        rays.data[ray_index].origin = pixel_data.world_pos + N * 0.01;
        rays.data[ray_index].direction = ray_dir;
        rays.data[ray_index].throughput = material_info.c_diff;
        rays.data[ray_index].t_max = MAX_TRACING_DISTANCE;
        rays.data[ray_index].t_min = 0.001;
        rays.data[ray_index].pixel_index = ray_index;
        rays.data[ray_index].ray_type = RAY_TYPE_DIFFUSE_BOUNCE;
        rays.data[ray_index].flags = RAY_FLAG_ACTIVE;
    } else {
        // No hit - mark ray as terminated
        let ray_index = pixel.y * DEFAULT_WIDTH + pixel.x;
        rays.data[ray_index].t_max = -1.0;
        rays.data[ray_index].flags = RAY_FLAG_TERMINATED;
    }

    // Write direct lighting
    textureStore(direct_lighting_texture, pixel, vec4<f32>(direct_light, 1.0));
}
