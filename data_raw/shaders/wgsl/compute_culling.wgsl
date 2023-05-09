#import "common.inc"
#import "utils.inc"

struct CullingData {
    view: mat4x4<f32>,
    mesh_flags: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> culling_data: CullingData;
@group(0) @binding(2)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(3)
var<storage, read> meshlets_culling: MeshletsCulling;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> bhv: BHV;
@group(0) @binding(6)
var<storage, read> meshes_flags: MeshFlags;

@group(1) @binding(0)
var<storage, read_write> count: atomic<u32>;
@group(1) @binding(1)
var<storage, read_write> commands: DrawIndexedCommands;
@group(1) @binding(2)
var<storage, read_write> culling_result: array<atomic<u32>>;

#import "matrix_utils.inc"


//ScreenSpace Frustum Culling
fn is_sphere_inside_frustum(center: vec3<f32>, radius: f32, frustum: array<vec4<f32>, 4>) -> bool {
    var visible: bool = true;    
    var f = frustum;
    for(var i = 0; i < 4; i = i + 1) {  
        visible = visible && !(dot(f[i].xyz, center) + f[i].w + radius <= 0.);
    }   
    return visible;
}

fn is_box_inside_frustum(min: vec3<f32>, max: vec3<f32>, frustum: array<vec4<f32>, 4>) -> bool {
    var visible: bool = false;    
    var points: array<vec3<f32>, 8>;
    points[0] = min;
    points[1] = max;
    points[2] = vec3<f32>(min.x, min.y, max.z);
    points[3] = vec3<f32>(min.x, max.y, max.z);
    points[4] = vec3<f32>(min.x, max.y, min.z);
    points[5] = vec3<f32>(max.x, min.y, min.z);
    points[6] = vec3<f32>(max.x, max.y, min.z);
    points[7] = vec3<f32>(max.x, min.y, max.z);
      
    var f = frustum;
    for(var i = 0; !visible && i < 4; i = i + 1) {  
        for(var p = 0; !visible && p < 8; p = p + 1) {        
            visible = visible || !(dot(f[i].xyz, points[p]) + f[i].w <= 0.);
        }
    }   
    return visible;
}

fn is_cone_visible(center: vec3<f32>, cone_axis: vec3<f32>, cone_cutoff: f32, radius: f32) -> bool {
    let direction = center - culling_data.view[3].xyz;
    return dot(normalize(direction), cone_axis) < (cone_cutoff * length(direction) + radius);
}


@compute
@workgroup_size(32, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let total = arrayLength(&meshlets.data);
    let meshlet_id = global_invocation_id.x;
    if (meshlet_id >= total) {
        return;
    }
    let command = &commands.data[meshlet_id];
    (*command).vertex_count = 0u;
    (*command).instance_count = 0u;
    (*command).base_index = 0u;
    (*command).vertex_offset = 0;
    (*command).base_instance = 0u;

    let meshlet = &meshlets.data[meshlet_id];
    let mesh_id = (*meshlet).mesh_index;
    
    if (meshes_flags.data[mesh_id] != culling_data.mesh_flags) {
        return;        
    }

    let mesh = &meshes.data[mesh_id];
    let bb_id = (*mesh).blas_index + (*meshlet).blas_index;
    let bb = &bhv.data[bb_id];
    let max = transform_vector((*bb).max, (*mesh).position, (*mesh).orientation, (*mesh).scale);
    let min = transform_vector((*bb).min, (*mesh).position, (*mesh).orientation, (*mesh).scale);
    let d = (max-min) * 0.5;
    let center = min + d;
    let radius = length(d);

    let mvp = constant_data.proj * culling_data.view;
    let row0 = matrix_row(mvp, 0u);
    let row1 = matrix_row(mvp, 1u);
    let row3 = matrix_row(mvp, 3u);

    var frustum: array<vec4<f32>, 4>;
    frustum[0] = normalize_plane(row3 + row0);
    frustum[1] = normalize_plane(row3 - row0);
    frustum[2] = normalize_plane(row3 + row1);
    frustum[3] = normalize_plane(row3 - row1);


    if !is_sphere_inside_frustum(center, radius, frustum) {
        return;
    }
    
    if !is_box_inside_frustum(min, max, frustum) {
        return;
    }

    let cone_culling = &meshlets_culling.data[meshlet_id];
    let cone_axis_cutoff = unpack4x8snorm((*cone_culling).cone_axis_cutoff);
    let cone_axis = rotate_vector(cone_axis_cutoff.xyz, (*mesh).orientation);    
    if (is_cone_visible((*cone_culling).center, cone_axis, cone_axis_cutoff.w, radius))
    {
        atomicAdd(&count, 1u);
        let draw_group_index = workgroup_id.x;
        atomicOr(&culling_result[draw_group_index], 1u << local_invocation_id.x);
    }
}