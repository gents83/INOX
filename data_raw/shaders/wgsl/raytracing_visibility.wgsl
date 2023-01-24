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

@group(1) @binding(0)
var<storage, read> tlas: BHV;
@group(1) @binding(1)
var<storage, read> bhv: BHV;
@group(1) @binding(2)
var<storage, read> meshes_inverse_matrix: Matrices;
@group(1) @binding(3)
var<storage, read> frame_index: FrameIndex;

@group(2) @binding(0)
var render_target: texture_storage_2d<rgba8unorm, read_write>;

#import "matrix_utils.inc"
#import "raytracing.inc"


fn frame_origin(frame_index: u32, width: u32, height: u32) -> vec2<u32>
{
    let step_x = width / 2u;
    let step_y = height / 2u;
    var origin = vec2<u32>((frame_index >> 1u) & 1u, frame_index & 1u);
    origin.x = origin.x * step_x;
    origin.y = origin.y * step_y;
    return origin;
}

@compute
@workgroup_size(8, 8, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = vec2<u32>(textureDimensions(render_target));
    let frame_origin = frame_origin(frame_index.data[0], dimensions.x, dimensions.y);

    let pixel = vec2<u32>(workgroup_id.x * 8u + local_invocation_id.x + frame_origin.x, 
                          workgroup_id.y * 8u + local_invocation_id.y + frame_origin.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y)
    {
        return;
    }    
    // Create a ray with the current fragment as the origin.
    let ray = compute_ray(pixel, dimensions);
    var nearest = MAX_FLOAT;  
    var visibility_id = 0u;
    
    var tlas_index = 0;
    
    while (tlas_index >= 0)
    {
        let node = &tlas.data[u32(tlas_index)];    
        let intersection = intersect_aabb(ray, (*node).min, (*node).max);
        if (intersection < nearest) {
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
            let transformed_ray = Ray(transformed_origin.xyz, transformed_direction.xyz);
            let result = traverse_bhv_of_meshlets(transformed_ray, mesh_id, nearest);
            if (result.visibility_id > 0u && result.distance < nearest) {
                visibility_id = result.visibility_id;
                nearest = result.distance;
            }
        }
        tlas_index = (*node).miss;
    } 
    
    textureStore(render_target, vec2<i32>(pixel), unpack4x8unorm(visibility_id));
}