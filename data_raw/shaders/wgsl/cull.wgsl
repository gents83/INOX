
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


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> meshlets: array<MeshletData>;

fn cone_culling(meshlet: MeshletData, camera_position: vec3<f32>) -> bool {
    let direction = meshlet.center - camera_position;
    return dot(direction, meshlet.cone_axis) >= meshlet.cone_cutoff * length(direction) + meshlet.radius;
}

@compute
@workgroup_size(32, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&meshlets);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }
}