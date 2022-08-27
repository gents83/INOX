#import "utils.wgsl"
#import "common.wgsl"


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> meshlets_bb: AABBs;
@group(1) @binding(0)
var<storage, read_write> count: atomic<u32>;
@group(1) @binding(1)
var<storage, read_write> commands: DrawIndexedCommands;

//ScreenSpace Frustum Culling
fn is_inside_frustum(center: vec3<f32>, radius: f32) -> bool {
    let mvp = constant_data.proj * constant_data.view;
    let row0 = matrix_row(mvp, 0u);
    let row1 = matrix_row(mvp, 1u);
    let row3 = matrix_row(mvp, 3u);
    let frustum_1 = normalize_plane(row3 + row0);
    let frustum_2 = normalize_plane(row3 - row0);
    let frustum_3 = normalize_plane(row3 + row1);
    let frustum_4 = normalize_plane(row3 - row1);
    var visible: bool = true;    
    visible = visible && (dot(frustum_1.xyz, center) + frustum_1.w + radius > 0.);
    visible = visible && (dot(frustum_2.xyz, center) + frustum_2.w + radius > 0.);
    visible = visible && (dot(frustum_3.xyz, center) + frustum_3.w + radius > 0.);
    visible = visible && (dot(frustum_4.xyz, center) + frustum_4.w + radius > 0.);    
    return visible;
}

//fn is_cone_culled(center: vec3<f32>, radius: f32, cone_axis: vec3<f32>, cone_cutoff: f32, orientation: vec4<f32>, camera_position: vec3<f32>) -> bool {
//    let axis = rotate_vector(cone_axis, orientation);
//
//    let direction = center - camera_position;
//    return dot(direction, axis) < cone_cutoff * length(direction) + radius;
//}


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
    let view_pos = constant_data.view[3].xyz;

    if (is_inside_frustum(center, radius)) 
    {
        let index = atomicAdd(&count, 1u);
        let command = &commands.data[index];
        (*command).vertex_count = (*meshlet).indices_count;
        (*command).instance_count = 1u;
        (*command).base_index = (*mesh).indices_offset + (*meshlet).indices_offset;
        (*command).vertex_offset = i32((*mesh).vertex_offset);
        (*command).base_instance = meshlet_id;
    } 
    
    //let cone_axis = vec3<f32>((*meshlet).cone_axis[0], (*meshlet).cone_axis[1], (*meshlet).cone_axis[2]);
    //is_visible = is_cone_culled(center.xyz, radius, cone_axis, (*meshlet).cone_cutoff, meshes.meshes[mesh_index].orientation, view_pos);
    //if (!is_visible) {
    //    (*command).vertex_count = 0u;
    //    (*command).instance_count = 0u;
    //    return;
    //}
}