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
var<storage, read> culling_result: array<atomic<u32>>;
@group(0) @binding(5)
var<storage, read> tlas: BHV;
@group(0) @binding(6)
var<storage, read> blas: BHV;
@group(0) @binding(7)
var<storage, read> meshes_inverse_matrix: Matrices;

@group(1) @binding(0)
var<storage, read_write> rays: Rays;
@group(1) @binding(1)
var<storage, read_write> jobs_count: atomic<i32>;
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
        let node = &tlas.data[u32(tlas_index)];    
        let intersection = intersect_aabb(&ray, (*node).min, (*node).max);
        if (intersection >= nearest) {
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


@compute
@workgroup_size(8, 8, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = vec2<u32>(textureDimensions(render_target));
    var job_index = atomicSub(&jobs_count, 1) - 1;
    while(job_index >= 0)
    {
        let v = execute_job(u32(job_index));
        let x = job_index % i32(dimensions.x);
        let y = job_index / i32(dimensions.x);
        textureStore(render_target, vec2<i32>(x, y), v);
        job_index = atomicSub(&jobs_count, 1) - 1;
    }
}