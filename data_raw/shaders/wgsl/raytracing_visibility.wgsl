#import "utils.inc"
#import "common.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
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
var<storage, read> tlas: BHV;
@group(0) @binding(7)
var<storage, read> bhv: BHV;
@group(0) @binding(8)
var<storage, read> meshes_inverse_matrix: Matrices;

@group(1) @binding(0)
var render_target: texture_storage_2d<rgba8unorm, read_write>;

#import "matrix_utils.inc"
#import "raytracing.inc"


@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = vec2<u32>(textureDimensions(render_target));
         
    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
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
            if ((*node).reference >= 0) {
                //leaf node
                let mesh_id = u32((*node).reference);
                let inverse_matrix = &meshes_inverse_matrix.data[mesh_id];    
                let transformed_origin = (*inverse_matrix) * vec4<f32>(ray.origin, 1.);
                let transformed_direction = (*inverse_matrix) * vec4<f32>(ray.direction, 0.);
                let transformed_ray = Ray(transformed_origin.xyz, transformed_direction.xyz);
                let result = traverse_bhv(transformed_ray, mesh_id);
                if (result.visibility_id > 0u && result.distance < nearest) {
                    visibility_id = result.visibility_id;
                    nearest = result.distance;
                }
            } else {
                //inner node
                tlas_index = tlas_index + 1;
                continue;
            }
        }
        tlas_index = (*node).miss;
    } 
    //if (visibility_id > 0u) {
    //    visibility_id = 0xFFFFFFFFu;
    //}
    textureStore(render_target, vec2<i32>(pixel), unpack4x8unorm(visibility_id));
}