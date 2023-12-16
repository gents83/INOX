#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(0) @binding(3)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(6)
var<storage, read> culling_result: array<u32>;

@group(1) @binding(0)
var<storage, read> meshes_inverse_matrix: Matrices;
@group(1) @binding(1)
var<storage, read> materials: Materials;
@group(1) @binding(2)
var<storage, read> textures: Textures;
@group(1) @binding(3)
var<storage, read> bhv: BHV;
@group(1) @binding(4)
var<uniform> tlas_starting_index: u32;
@group(1) @binding(5)
var<storage, read_write> rays: Rays;
@group(1) @binding(6)
var render_target: texture_storage_2d<rgba8unorm, read_write>;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"

const MAX_CANDIDATES: i32 = 32;
var<workgroup> candidate_triangles: array<CandidateTriangleData, MAX_CANDIDATES>;

struct CandidateTriangleData {
    triangle_id: u32,
    meshlet_id: u32,
    meshlet_indices_offset: u32,
    vertices_position_offset: u32,
};

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
    var candidate_triangles_index = 0;
    
    while(hit_type != HIT_DATA_MISS)
    {
        if(node_index < 0) {
            if(hit_type == HIT_DATA_TLAS) {
                break;
            }
            let is_triangle = hit_type == HIT_DATA_TRIANGLE;
            node_index = select(tlas_sibling, blas_sibling, is_triangle);
            hit_type = select(HIT_DATA_TLAS, HIT_DATA_BLAS, is_triangle);
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
            candidate_triangles[candidate_triangles_index] = CandidateTriangleData(u32(node.reference), meshlet_id, meshlet_indices_offset, vertices_position_offset);
            candidate_triangles_index += 1;
        }
        else if(hit_type == HIT_DATA_BLAS) {  
            hit_type = HIT_DATA_TRIANGLE;
            blas_sibling = node.miss;
            meshlet_id = meshlets_offset + u32(node.reference);  
            let index = meshlet_id / 32u;
            let offset = meshlet_id - (index * 32u);
            let is_meshlet_visible =  (culling_result[index] & (1u << offset)) > 0u;   
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

            let inverse_matrix = &meshes_inverse_matrix.data[mesh_id];    
            let local_ray_origin = (*inverse_matrix) * vec4<f32>(world_ray.origin, 1.);
            let local_ray_direction = (*inverse_matrix) * vec4<f32>(world_ray.direction, 0.);
            ray.origin = local_ray_origin.xyz;
            ray.direction = local_ray_direction.xyz;
        }
    }
    
    for(var i: i32 = 0; i < candidate_triangles_index; i++) {
        let triangle_id = u32(candidate_triangles[i].triangle_id);
        let index_offset = candidate_triangles[i].meshlet_indices_offset + (triangle_id * 3u);
        let v1 = runtime_vertices.data[candidate_triangles[i].vertices_position_offset + indices.data[index_offset]].world_pos;
        let v2 = runtime_vertices.data[candidate_triangles[i].vertices_position_offset + indices.data[index_offset + 1u]].world_pos;
        let v3 = runtime_vertices.data[candidate_triangles[i].vertices_position_offset + indices.data[index_offset + 2u]].world_pos;
        let distance = intersect_triangle(world_ray, v1, v2, v3);
        visibility_id = select(visibility_id, ((candidate_triangles[i].meshlet_id + 1u) << 8u) | triangle_id, distance < hit_distance);
        hit_distance = min(hit_distance, distance);
    }
    return Result(hit_distance, visibility_id);
}


fn execute_job(job_index: u32, pixel: vec2<f32>, dimensions: vec2<f32>, mvp: mat4x4<f32>) -> vec4<f32>  
{    
    var ray = rays.data[job_index];
    var seed = vec2<u32>(pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);
    var uv_coords = 2. * (pixel / dimensions) - vec2<f32>(1., 1.);
    uv_coords.y = -uv_coords.y;
    var pixel_color = vec3<f32>(0.);
    for (var sample = 0u; sample < NUM_SAMPLES_PER_PIXEL; sample++) {
        var radiance_data = RadianceData(ray, seed, vec3<f32>(0.), vec3<f32>(1.));
        for (var bounce = 0u; bounce < MAX_PATH_BOUNCES; bounce++) {
            let result = traverse_bvh(radiance_data.ray, i32(tlas_starting_index));            
            if (result.visibility_id == 0u) {
                break;
            }
            radiance_data.ray.t_max = result.distance;  
            radiance_data = compute_radiance_from_visibility(result.visibility_id, uv_coords, radiance_data, mvp); 
            seed = radiance_data.seed;
        }
        pixel_color += radiance_data.radiance;
    }
    pixel_color /= f32(NUM_SAMPLES_PER_PIXEL);
    return vec4<f32>(pixel_color, 1.);
}


const MAX_WORKGROUP_SIZE: i32 = 16*16;
const MAX_SIZE: u32 = u32(MAX_WORKGROUP_SIZE);

var<workgroup> jobs_count: atomic<i32>;

const GAMMA:f32 = 2.2;

fn Uncharted2ToneMapping(color: vec3<f32>) -> vec3<f32> {
	let A = 0.15;
	let B = 0.50;
	let C = 0.10;
	let D = 0.20;
	let E = 0.02;
	let F = 0.30;
	let W = 11.2;
	let exposure = 2.;
	var result = color * exposure;
	result = ((result * (A * result + C * B) + D * E) / (result * (A * result + B) + D * F)) - E / F;
	let white = ((W * (A * W + C * B) + D * E) / (W * (A * W + B) + D * F)) - E / F;
	result /= white;
	result = pow(result, vec3<f32>(1. / GAMMA));
	return result;
}

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let max_jobs = 16 * 16;
    let group = vec2<i32>(i32(workgroup_id.x), i32(workgroup_id.y));
    let dimensions = vec2<i32>(textureDimensions(render_target));
    atomicStore(&jobs_count, max_jobs);
    
    let mvp = constant_data.proj * constant_data.view;
    var job_index = 0;
    while(job_index < MAX_WORKGROUP_SIZE)
    {
        let pixel = vec2<i32>(group.x * 16 + job_index % 16, 
                              group.y * 16 + job_index / 16);
        if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
            job_index = max_jobs - atomicSub(&jobs_count, 1);
            continue;
        }    
        
        let index = u32(pixel.y * dimensions.x + pixel.x);

        var out_color = execute_job(index, vec2<f32>(pixel), vec2<f32>(dimensions), mvp);
        out_color = vec4<f32>(Uncharted2ToneMapping(out_color.rgb), 1.);
        if(constant_data.frame_index > 0u) {
            var prev_value = textureLoad(render_target, pixel);
            let weight = 1. / f32(constant_data.frame_index + 1u);
            out_color = mix(prev_value, out_color, weight);
        } 
        textureStore(render_target, pixel, out_color);
        job_index = max_jobs - atomicSub(&jobs_count, 1);
    }
}