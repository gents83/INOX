#import "utils.wgsl"
#import "common.wgsl"


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(2)
var<storage, read> matrices: Matrices;
@group(0) @binding(3)
var<storage, read> instances: Instances;
@group(0) @binding(4)
var<storage, read_write> commands: Commands;


fn transform_vector(v: vec3<f32>, q: vec4<f32>) -> vec3<f32> {
    return v + 2. * cross(q.xyz, cross(q.xyz, v) + q.w * v);
}

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
//    let axis = transform_vector(cone_axis, orientation);
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
    let meshlet_index = global_invocation_id.x;
    if (meshlet_index >= total) {
        return;
    }
    let command = &commands.data[meshlet_index];
    let instance_index = (*command).base_instance;
    let matrix_index = instances.data[instance_index].matrix_index;
    let m = &matrices.data[matrix_index];
    let meshlet = &meshlets.data[meshlet_index];

    let scale = extract_scale((*m));
    let center = (*m) * vec4<f32>((*meshlet).center_radius.xyz, 1.0);
    let radius = (*meshlet).center_radius.w * scale.x;
    let view_pos = constant_data.view[3].xyz;

    var is_visible = is_inside_frustum(center.xyz, radius);
    if (!is_visible) {
        (*command).vertex_count = 0u;
        (*command).instance_count = 0u;
        return;
    }
    //let cone_axis = vec3<f32>((*meshlet).cone_axis[0], (*meshlet).cone_axis[1], (*meshlet).cone_axis[2]);
    //is_visible = is_cone_culled(center.xyz, radius, cone_axis, (*meshlet).cone_cutoff, meshes.meshes[mesh_index].orientation, view_pos);
    //if (!is_visible) {
    //    (*command).vertex_count = 0u;
    //    (*command).instance_count = 0u;
    //    return;
    //}
}