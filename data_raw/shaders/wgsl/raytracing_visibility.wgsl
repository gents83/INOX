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
var<storage, read> bhv: BHV;

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

    let mesh_count = arrayLength(&meshes.data);    
    for (var mesh_id = 0u; mesh_id < mesh_count; mesh_id++) {
        let result = traverse_bhv(ray, mesh_id);
        if (result.visibility_id > 0u && result.distance < nearest) {
            visibility_id = result.visibility_id;
            nearest = result.distance;
        }
    }    
    //if (visibility_id > 0u) {
    //    visibility_id = 0xFFFFFFFFu;
    //}
    textureStore(render_target, vec2<i32>(pixel), unpack4x8unorm(visibility_id));
}