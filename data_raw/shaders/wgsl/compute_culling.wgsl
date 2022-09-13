

#import "utils.inc"
#import "common.inc"

struct CullingData {
    view: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> culling_data: CullingData;
@group(0) @binding(2)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(3)
var<storage, read> meshes: Meshes;
@group(0) @binding(4)
var<storage, read> meshlets_bb: AABBs;

@group(1) @binding(0)
var<storage, read_write> count: atomic<u32>;
@group(1) @binding(1)
var<storage, read_write> commands: DrawIndexedCommands;


//ScreenSpace Frustum Culling
fn is_sphere_inside_frustum(center: vec3<f32>, radius: f32, frustum: array<vec4<f32>, 4>) -> bool {
    var visible: bool = true;    
    var f = frustum;
    for(var i = 0; i < 4; i = i + 1) {  
        visible = visible && (dot(f[i].xyz, center) + f[i].w + radius > 0.);
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
            visible = visible || (dot(f[i].xyz, points[p]) + f[i].w > 0.);
        }
    }   
    return visible;
}

fn is_cone_culled(center: vec3<f32>, radius: f32, cone_axis: vec3<f32>, cone_cutoff: f32) -> bool {
    let direction = center - culling_data.view[3].xyz;
    return dot(direction, cone_axis) < cone_cutoff * length(direction) + radius;
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
    let meshlet = &meshlets.data[meshlet_id];
    let mesh_id = (*meshlet).mesh_index;
    let mesh = &meshes.data[mesh_id];
    let bb_id = (*meshlet).aabb_index;
    let bb = &meshlets_bb.data[bb_id];

    let radius = abs((*bb).max-(*bb).min) * 0.5;
    let center_bb = (*bb).min + radius;
    let center = transform_vector(center_bb, (*mesh).position, (*mesh).orientation, (*mesh).scale);
    let radius = length(radius * (*mesh).scale);

    let mvp = constant_data.proj * culling_data.view;
    let row0 = matrix_row(mvp, 0u);
    let row1 = matrix_row(mvp, 1u);
    let row3 = matrix_row(mvp, 3u);

    var frustum: array<vec4<f32>, 4>;
    frustum[0] = normalize_plane(row3 + row0);
    frustum[1] = normalize_plane(row3 - row0);
    frustum[2] = normalize_plane(row3 + row1);
    frustum[3] = normalize_plane(row3 - row1);

    var is_visible = is_sphere_inside_frustum(center, radius, frustum);
    if !is_visible {
        return;
    }
    let max = transform_vector((*bb).max, (*mesh).position, (*mesh).orientation, (*mesh).scale);
    let min = transform_vector((*bb).min, (*mesh).position, (*mesh).orientation, (*mesh).scale);
    is_visible = is_visible && is_box_inside_frustum(min, max, frustum); 
    if !is_visible {
        return;
    }
    
    let cone_axis = rotate_vector((*meshlet).cone_axis_cutoff.xyz, (*mesh).orientation);
    is_visible = is_visible && is_cone_culled(center, radius, cone_axis, (*meshlet).cone_axis_cutoff.w);
    
    if (is_visible)
    {
        let index = atomicAdd(&count, 1u) - 1u;
        let command = &commands.data[index];
        (*command).vertex_count = (*meshlet).indices_count;
        (*command).instance_count = 1u;
        (*command).base_index = (*mesh).indices_offset + (*meshlet).indices_offset;
        (*command).vertex_offset = i32((*mesh).vertex_offset);
        (*command).base_instance = meshlet_id;
    }
}