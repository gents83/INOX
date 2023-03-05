#import "utils.inc"
#import "common.inc"

struct Data {
    width: u32,
    height: u32,
};


@group(0) @binding(0)
var<uniform> data: Data;
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
var<storage, read> rays: Rays;

#import "raytracing.inc"


fn execute_job(job_index: u32, dimensions: vec2<u32>) -> vec4<f32> {    
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
    return unpack4x8unorm(visibility_id);
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    //only one triangle, exceeding the viewport size
    let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
    let pos = vec4<f32>(uv * vec2<f32>(2., -2.) + vec2<f32>(-1., 1.), 0., 1.);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = pos;
    vertex_out.uv = uv;
    return vertex_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = vec2<u32>(u32(v_in.uv.x * f32(data.width)), u32(v_in.uv.y * f32(data.height)));

    let total_job_index = pixel.y * data.width + pixel.x;
    
    let texture_color = execute_job(total_job_index, vec2<u32>(data.width, data.height));
    return texture_color;
}