#import "utils.inc"
#import "common.inc"

struct FrameIndex {
    data: array<u32>,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices: Vertices;
@group(0) @binding(3)
var<storage, read> positions: Positions;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(6)
var<storage, read> meshlets_culling: MeshletsCulling;
@group(0) @binding(7)
var<storage, read> culling_result: array<atomic<u32>>;

@group(1) @binding(0)
var<storage, read> tlas: BHV;
@group(1) @binding(1)
var<storage, read> bhv: BHV;
@group(1) @binding(2)
var<storage, read> meshes_inverse_matrix: Matrices;
@group(1) @binding(3)
var<storage, read_write> rays: Rays;
@group(1) @binding(4)
var<storage, read_write> jobs: array<atomic<u32>>;

@group(2) @binding(0)
var render_target: texture_storage_2d<rgba8unorm, write>;

#import "matrix_utils.inc"
#import "jobs.inc"
#import "raytracing.inc"


fn execute_job(job_index: u32, dimensions: vec2<u32>) {    
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
        let result = traverse_bhv_of_meshlets(&transformed_ray, mesh_id, nearest);
        visibility_id = select(visibility_id, result.visibility_id, result.distance < nearest);
        nearest = result.distance;
        tlas_index = (*node).miss;
    } 
        
    let x = i32(job_index % dimensions.x);
    let y = i32(job_index / dimensions.x); 
    textureStore(render_target, vec2<i32>(x, y), unpack4x8unorm(visibility_id));
}

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = vec2<u32>(textureDimensions(render_target));
    let atomic_count = arrayLength(&jobs);
    
    let pixel = vec2<u32>(workgroup_id.x * 16u + local_invocation_id.x, 
                          workgroup_id.y * 16u + local_invocation_id.y);
    let total_job_index = pixel.y * dimensions.x + pixel.x;
    execute_job(total_job_index, dimensions);
    /*
    var atomic_index = i32(total_job_index / 32u);
    let last_atomic = atomic_index + (16 * 16) / 32;
    while (atomic_index >= 0 && atomic_index < last_atomic) {
        let job_index = extract_job_index(u32(atomic_index));
        if (job_index < 0) {
            atomic_index = find_busy_atomic(u32(atomic_index), u32(last_atomic));
            continue;
        }
        execute_job(u32(atomic_index * 32 + job_index), dimensions);
    }
    */
}