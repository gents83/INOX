#import "common.inc"
#import "utils.inc"

struct Job {
    state: u32,
    tlas_index: i32,
    blas_index: i32,
    nearest: f32,
    visibility_id: u32,
};

@group(0) @binding(0)
var<storage, read> indices: Indices;
@group(0) @binding(1)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(4)
var<storage, read> culling_result: array<u32>;
@group(0) @binding(5)
var<storage, read> bhv: BHV;
@group(0) @binding(6)
var<storage, read> meshes_inverse_matrix: Matrices;

@group(1) @binding(0)
var<uniform> tlas_starting_index: u32;
@group(1) @binding(1)
var<storage, read_write> rays: Rays;
@group(1) @binding(2)
var render_target: texture_storage_2d<rgba8unorm, write>;

#import "matrix_utils.inc"
#import "raytracing.inc"


fn execute_job(job_index: u32) -> vec4<f32>  {    
    var ray = rays.data[job_index];
    var nearest = MAX_FLOAT;  
    var visibility_id = 0u;
    
    var tlas_index = 0;
    
    while (tlas_index >= 0)
    {
        let node = &bhv.data[tlas_starting_index + u32(tlas_index)];    
        let intersection = intersect_aabb(&ray, (*node).min, (*node).max);
        if (intersection > nearest) {
            tlas_index = (*node).miss;
            continue;
        }
        if ((*node).reference < 0) {
            //inner node
            tlas_index = tlas_index + 1;
            continue;
        }
        //leaf node
        let mesh_id = u32((*node).reference);
        let inverse_matrix = &meshes_inverse_matrix.data[mesh_id];    
        let transformed_origin = (*inverse_matrix) * vec4<f32>(ray.origin, 1.);
        let transformed_direction = (*inverse_matrix) * vec4<f32>(ray.direction, 0.);
        var transformed_ray = Ray(transformed_origin.xyz, ray.t_min, transformed_direction.xyz, ray.t_max);
        let result = traverse_bhv_of_meshlets(&ray, &transformed_ray, mesh_id, nearest);
        visibility_id = select(visibility_id, result.visibility_id, result.distance < nearest);
        nearest = result.distance;
        tlas_index = (*node).miss;
    } 
    return unpack4x8unorm(visibility_id);
}

var<workgroup> jobs_count: atomic<i32>;

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
    
    var job_index = 0;
    while(job_index < max_jobs)
    {
        let pixel = vec2<i32>(group.x * 16 + job_index % 16, 
                              group.y * 16 + job_index / 16);
        if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
            job_index = max_jobs - atomicSub(&jobs_count, 1);
            continue;
        }    
        
        let v = execute_job(u32(pixel.y * dimensions.x + pixel.x));
        textureStore(render_target, pixel, v);
        job_index = max_jobs - atomicSub(&jobs_count, 1);
    }
}