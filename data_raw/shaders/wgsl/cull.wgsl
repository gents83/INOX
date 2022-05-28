
struct ConstantData {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    screen_width: f32,
    screen_height: f32,
    flags: u32,
};


struct MeshletData {
    center: vec3<f32>,
    radius: f32,
    cone_axis: vec3<f32>,
    cone_cutoff: f32,
    vertices_count: u32,
    vertices_offset: u32,
    indices_count: u32,
    indices_offset: u32,
};

struct CullPassData {
    meshlets: array<MeshletData>,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> cull_pass_data: CullPassData;

fn cone_culling(meshlet: MeshletData, camera_position: vec3<f32>) -> bool {
    let direction = meshlet.center - camera_position;
    return dot(direction, meshlet.cone_axis) >= meshlet.cone_cutoff * length(direction) + meshlet.radius;
}

@compute
@workgroup_size(32, 1, 1)
fn main(@builtin(local_invocation_id) local_invocation_id: vec3<u32>, @builtin(local_invocation_index) local_invocation_index: u32, @builtin(global_invocation_id) global_invocation_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let total = arrayLength(&cull_pass_data.meshlets);
    let local_id = local_invocation_id.x;
    let local_index = local_invocation_index;
    let global_id = global_invocation_id.x;
    let wkgrp_id = workgroup_id.x;
    if (local_invocation_index >= total) {
        return;
    }
}