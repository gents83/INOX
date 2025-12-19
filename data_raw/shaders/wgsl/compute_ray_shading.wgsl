#import "common.inc"
#import "utils.inc"
#import "vertex_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "ray_data.inc"
#import "ray_types.inc"
#import "sampling.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

// Output textures for accumulating indirect lighting
@group(0) @binding(1)
var indirect_diffuse_texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2)
var indirect_specular_texture: texture_storage_2d<rgba16float, write>;

// Group 0: Ray Data (bindings 3-5 to make room for output textures at 1-2)
@group(0) @binding(3)
var<storage, read> rays: Rays;
@group(0) @binding(4)
var<storage, read> intersections: Intersections;
@group(0) @binding(5)
var<storage, read_write> rays_next: Rays;

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

// Materials and Lights
// Group 2: Materials, Textures, Lights
@group(2) @binding(0)
var<uniform> materials: Materials;
@group(2) @binding(1)
var<uniform> textures: Textures;
@group(2) @binding(2)
var<uniform> lights: Lights;

#import "texture_utils.inc"

const WORKGROUP_SIZE: u32 = 64u;

fn get_pixel_data_from_intersection(ray: PathRay, intersection: Intersection) -> PixelData {
    let hit_point = ray.origin + ray.direction * intersection.t;
    let u = intersection.u;
    let v = intersection.v;
    let w = 1.0 - u - v;
    let barycentrics = vec3<f32>(w, u, v);
    
    let primitive_index = u32(intersection.primitive_index);
    let index_offset = primitive_index * 3u;
    
    return get_pixel_data_from_triangle(
        u32(intersection.instance_id),
        index_offset,
        hit_point,
        barycentrics,
        true
    );
}

@compute
@workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let ray_index = global_invocation_id.x;
    if (ray_index >= arrayLength(&rays.data)) {
        return;
    }

    let ray = rays.data[ray_index];
    let intersection = intersections.data[ray_index];
    
    // Skip inactive or terminated rays
    if ((ray.flags & RAY_FLAG_ACTIVE) == 0u || (ray.flags & RAY_FLAG_TERMINATED) != 0u) {
        return;
    }
    
    // Check for miss
    if (intersection.instance_id < 0) {
        // Environment contribution (if any)
        let pixel_index = ray.pixel_index;
        let dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let pixel = vec2<u32>(pixel_index % dimensions.x, pixel_index / dimensions.x);
        
        // For now, just terminate the ray
        rays_next.data[ray_index].t_max = -1.0;
        rays_next.data[ray_index].flags = RAY_FLAG_TERMINATED;
        return;
    }

    // Hit processing
    var pixel_data = get_pixel_data_from_intersection(ray, intersection);
    
    // Compute lighting at hit point
    let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);
    let light_contribution = material_info.f_color.rgb;
    let radiance = material_info.f_color.rgb;
    
    // Apply throughput
    let contribution = radiance * ray.throughput;
    let pixel_index = ray.pixel_index;
    let pixel = vec2<u32>(pixel_index % DEFAULT_WIDTH, pixel_index / DEFAULT_WIDTH);
    
    // Write contribution to appropriate output texture based on ray type
    if (ray.ray_type == RAY_TYPE_DIFFUSE_BOUNCE) {
        textureStore(indirect_diffuse_texture, pixel, vec4<f32>(contribution, 0.0));
    } else {
        textureStore(indirect_specular_texture, pixel, vec4<f32>(contribution, 0.0));
    }
    
    // Generate next bounce (simplified - could be more sophisticated)
    var seed = vec2<u32>(pixel_index, constant_data.frame_index + ray_index);
    let rnd = get_random_numbers(&seed);
    let rnd_dir = get_random_numbers(&seed);
    
    let cam_pos = constant_data.inv_view[3].xyz;
    let V = normalize(cam_pos - pixel_data.world_pos);
    
    let diffuse_weight = length(material_info.c_diff);
    let specular_weight = length(material_info.f0);
    let total_weight = diffuse_weight + specular_weight;
    
    if (total_weight > 0.001 && length(ray.throughput) > 0.01) {
        let p_specular = specular_weight / total_weight;
        
        let N = normalize(material_info.shading_normal);
        let up = select(vec3(1., 0., 0.), vec3(0., 1., 0.), abs(N.z) < 0.999);
        let tangent = normalize(cross(up, N));
        let bitangent = cross(N, tangent);
        let tbn_sample = mat3x3<f32>(tangent, bitangent, N);
        
        var next_dir: vec3<f32>;
        var throughput_adj: vec3<f32>;
        var next_ray_type: u32;
        
        if (rnd.x < p_specular) {
            let H = importance_sample_ggx(material_info.perceptual_roughness, rnd_dir, N, V);
            next_dir = normalize(reflect(-V, H));
            throughput_adj = material_info.f0 / p_specular;
            next_ray_type = RAY_TYPE_SPECULAR_BOUNCE;
        } else {
            let local_dir = sample_cosine_weighted_hemisphere(rnd_dir);
            next_dir = normalize(tbn_sample * local_dir);
            throughput_adj = material_info.c_diff / (1.0 - p_specular);
            next_ray_type = RAY_TYPE_DIFFUSE_BOUNCE;
        }
        
        rays_next.data[ray_index].origin = pixel_data.world_pos + next_dir * 0.01;
        rays_next.data[ray_index].direction = next_dir;
        rays_next.data[ray_index].throughput = ray.throughput * throughput_adj;
        rays_next.data[ray_index].t_max = MAX_TRACING_DISTANCE;
        rays_next.data[ray_index].t_min = 0.001;
        rays_next.data[ray_index].pixel_index = pixel_index;
        rays_next.data[ray_index].ray_type = next_ray_type;
        rays_next.data[ray_index].flags = RAY_FLAG_ACTIVE;
    } else {
        // Terminate ray
        rays_next.data[ray_index].t_max = -1.0;
        rays_next.data[ray_index].flags = RAY_FLAG_TERMINATED;
    }
}
