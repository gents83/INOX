#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(3)
var<storage, read> meshes: Meshes;
@group(0) @binding(4)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(5)
var<storage, read> materials: Materials;
@group(0) @binding(6)
var<storage, read> textures: Textures;
@group(0) @binding(7)
var<storage, read> lights: Lights;

@group(1) @binding(0)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(1) @binding(1)
var<storage, read> culling_result: array<u32>;
@group(1) @binding(2)
var<storage, read> bhv: BHV;
@group(1) @binding(3)
var<storage, read_write> rays: Rays;

@group(1) @binding(4)
var render_target: texture_storage_2d<rgba8unorm, read_write>;
@group(1) @binding(5)
var visibility_texture: texture_2d<f32>;
@group(1) @binding(6)
var depth_texture: texture_depth_2d;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"

//starting index = tlas_starting_index
fn traverse_bvh(r: Ray, tlas_starting_index: i32) -> Result {
    var world_ray = r;
    var ray = world_ray;
    var node_index = tlas_starting_index;
    var visibility_id = 0u;
    var hit_type = HIT_DATA_TLAS;
    var meshlets_offset = 0u;
    var meshlet_indices_offset = 0u;
    var vertices_position_offset = 0u;
    var meshlet_id = 0u;
    var blas_sibling = 0;
    var tlas_sibling = 0;
    var hit_distance = MAX_FLOAT;
    
    while(hit_type != HIT_DATA_MISS)
    {
        if(node_index < 0) {
            let is_tlas = hit_type == HIT_DATA_TLAS;
            let is_triangle = hit_type == HIT_DATA_TRIANGLE;
            node_index = select(tlas_sibling, blas_sibling, is_triangle);
            hit_type = select(HIT_DATA_TLAS, HIT_DATA_BLAS, is_triangle);
            hit_type = select(hit_type, HIT_DATA_MISS, is_tlas);
            ray.origin = select(world_ray.origin, ray.origin, is_triangle);
            ray.direction = select(world_ray.direction, ray.direction, is_triangle);
            continue;
        }
        
        let node = bhv.data[u32(node_index)]; 
        let intersection = intersect_aabb(&ray, node.min, node.max);        
        let is_miss = intersection > ray.t_max;
        let is_inner_node = node.reference < 0;
        node_index = select(node_index, node_index + 1, is_inner_node);
        node_index = select(node_index, node.miss, is_miss);
        if (is_miss || is_inner_node) {
            continue;
        }
        //leaf node
        if(hit_type == HIT_DATA_TRIANGLE) {     
            node_index = node.miss;        
            
            let triangle_id = u32(node.reference);
            let index_offset = meshlet_indices_offset + (triangle_id * 3u);
            let v1 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset]].world_pos;
            let v2 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset + 1u]].world_pos;
            let v3 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset + 2u]].world_pos;
            let distance = intersect_triangle(world_ray, v1, v2, v3);
            visibility_id = select(visibility_id, ((meshlet_id + 1u) << 8u) | triangle_id, distance < hit_distance);
            hit_distance = min(hit_distance, distance);
        }
        else if(hit_type == HIT_DATA_BLAS) {  
            hit_type = HIT_DATA_TRIANGLE;
            blas_sibling = node.miss;
            meshlet_id = meshlets_offset + u32(node.reference);  
            let index = meshlet_id / 32u;
            let offset = meshlet_id - (index * 32u);
            let is_meshlet_visible =  (culling_result[index] & (1u << offset)) != 0u;   
            let meshlet = meshlets.data[meshlet_id];
            meshlet_indices_offset = meshlet.indices_offset;
            node_index = select(blas_sibling, i32(meshlet.triangles_bhv_index), is_meshlet_visible); 
        }
        else {  
            hit_type = HIT_DATA_BLAS;
            tlas_sibling = node.miss;
            let mesh_id = u32(node.reference);
            let mesh = meshes.data[mesh_id];  
            meshlets_offset = mesh.meshlets_offset;
            vertices_position_offset = mesh.vertices_position_offset;
            node_index = i32(mesh.blas_index); 

            let matrix = transform_matrix(mesh.position, mesh.orientation, mesh.scale);    
            let inverse_matrix = matrix_inverse(matrix);
            let local_ray_origin = inverse_matrix * vec4<f32>(world_ray.origin, 1.);
            let local_ray_direction = inverse_matrix * vec4<f32>(world_ray.direction, 0.);
            ray.origin = local_ray_origin.xyz;
            ray.direction = local_ray_direction.xyz;
        }
    }
    return Result(hit_distance, visibility_id);
}

fn execute_job(job_index: u32, pixel: vec2<u32>, dimensions: vec2<u32>, mvp: mat4x4<f32>) -> vec4<f32>  
{    
    var ray = rays.data[job_index];
    var clip_coords = pixel_to_clip(pixel, dimensions);
    var pixel_color = vec3<f32>(0.);
    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index * 0xFFFFu);
    var radiance_data = RadianceData(ray.direction, vec3<f32>(0.), vec3<f32>(1.));
    
    let visibility_value = textureLoad(visibility_texture, pixel, 0);
    if((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_VISIBILITY_BUFFER) != 0) {
        return visibility_value;
    }
    let depth_value = textureLoad(depth_texture, pixel, 0);
    if((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER) != 0) {
        return vec4<f32>(1. - depth_value);
    }
    
    let visibility_id = pack4x8unorm(visibility_value);
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        return vec4<f32>(pixel_color, 1.);
    }
    if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS) != 0) 
    {
        let meshlet_color = hash((visibility_id >> 8u) + 1u);
        return vec4<f32>(vec3<f32>(
            f32(meshlet_color & 255u),
            f32((meshlet_color >> 8u) & 255u),
            f32((meshlet_color >> 16u) & 255u)
        ) / 255., 1.);
    }
    seed = get_random_numbers(seed);    
    radiance_data = compute_radiance_from_visibility(visibility_id, clip_coords, seed, radiance_data, mvp);     
    let hit_point = clip_to_world(clip_coords, depth_value);
    ray = Ray(hit_point + radiance_data.direction * HIT_EPSILON, HIT_EPSILON, radiance_data.direction, MAX_FLOAT);

    for (var bounce = 0u; bounce < MAX_PATH_BOUNCES; bounce++) {
        let result = traverse_bvh(ray, i32(constant_data.tlas_starting_index));  
        if (result.visibility_id == 0u || (result.visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
            break;
        }
        seed = get_random_numbers(seed);    
        radiance_data = compute_radiance_from_visibility(result.visibility_id, clip_coords, seed, radiance_data, mvp); 
        let hit_point = ray.origin + (ray.direction * result.distance);
        ray = Ray(hit_point + radiance_data.direction * HIT_EPSILON, HIT_EPSILON, radiance_data.direction, MAX_FLOAT);
    }
    pixel_color += radiance_data.radiance;
        
    return vec4<f32>(pixel_color, 1.);
}



const MAX_WORKGROUP_SIZE: u32 = 16u*16u;
var<workgroup> jobs_count: atomic<u32>;

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = textureDimensions(render_target);
    atomicStore(&jobs_count, MAX_WORKGROUP_SIZE);
    
    let mvp = constant_data.proj * constant_data.view;
    var job_index = 0u;
    while(job_index < MAX_WORKGROUP_SIZE)
    {
        let pixel = vec2<u32>(workgroup_id.x * 16u + job_index % 16u, 
                              workgroup_id.y * 16u + job_index / 16u);
        if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
            job_index = MAX_WORKGROUP_SIZE - atomicSub(&jobs_count, 1u);
            continue;
        }    
        
        let index = u32(pixel.y * dimensions.x + pixel.x);

        var out_color = execute_job(index, pixel, dimensions, mvp);
        out_color = vec4<f32>(Uncharted2ToneMapping(out_color.rgb), 1.);
        if(constant_data.frame_index > 0u) {
            var prev_value = textureLoad(render_target, pixel);
            let weight = 1. / f32(constant_data.frame_index + 1u);
            out_color = mix(prev_value, out_color, weight);
        } 
        textureStore(render_target, pixel, out_color);
        job_index = MAX_WORKGROUP_SIZE - atomicSub(&jobs_count, 1u);
    }
}