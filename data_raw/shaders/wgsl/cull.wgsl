
struct CullingPassData {
    cam_pos: vec3<f32>,
    padding: f32,
    frustum: array<vec4<f32>, 4>,
};

struct DrawCommand {
    vertex_count: u32,
    instance_count: u32,
    base_index: u32,
    vertex_offset: i32,
    base_instance: u32,
};

struct MeshData {
    position: vec3<f32>,
    scale: f32,
    orientation: vec4<f32>,
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

struct Meshlets {
    meshlets: array<MeshletData>,
};
struct Meshes {
    meshes: array<MeshData>,
};
struct Commands {
    commands: array<DrawCommand>,
};

@group(0) @binding(0)
var<uniform> cull_data: CullingPassData;
@group(0) @binding(1)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read_write> commands: Commands;


fn transform_vector(v: vec3<f32>, q: vec4<f32>) -> vec3<f32> {
    return v + 2. * cross(q.xyz, cross(q.xyz, v) + q.w * v);
}

//ScreenSpace Frustum Culling
fn is_inside_frustum(center: vec3<f32>, radius: f32) -> bool {
    var visible: bool = true;
    for (var i: i32 = 0; i < 4; i++) {
        visible = visible && (dot(cull_data.frustum[i].xyz, center) + cull_data.frustum[i].w * 2. + radius > 0.);
    }
    return visible;
}

fn is_cone_culled(meshlet: MeshletData, mesh: MeshData, camera_position: vec3<f32>) -> bool {
    let center = transform_vector(meshlet.center, mesh.orientation) * mesh.scale + mesh.position;
    let radius = meshlet.radius * mesh.scale;
    let cone_axis = transform_vector(vec3<f32>(meshlet.cone_axis[0], meshlet.cone_axis[1], meshlet.cone_axis[2]), mesh.orientation);
    let cone_cutoff = meshlet.cone_cutoff;

    let direction = center - camera_position;
    return dot(direction, cone_axis) < cone_cutoff * length(direction) + radius;
}


@compute
@workgroup_size(32, 1, 1)
fn main(@builtin(local_invocation_id) local_invocation_id: vec3<u32>, @builtin(local_invocation_index) local_invocation_index: u32, @builtin(global_invocation_id) global_invocation_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {
    let total = arrayLength(&meshlets.meshlets);
    let meshlet_index = global_invocation_id.x;
    if (meshlet_index >= total) {
        return;
    }
    let mesh_index = commands.commands[meshlet_index].base_instance;

    let center = transform_vector(meshlets.meshlets[meshlet_index].center, meshes.meshes[mesh_index].orientation) * meshes.meshes[mesh_index].scale + meshes.meshes[mesh_index].position;
    let radius = meshlets.meshlets[meshlet_index].radius * meshes.meshes[mesh_index].scale;

    if (!is_inside_frustum(center, radius)) {
        commands.commands[meshlet_index].vertex_count = 0u;
        commands.commands[meshlet_index].instance_count = 0u;
        return;
    }

    let cam_pos = cull_data.cam_pos;
    let is_visible = is_cone_culled(meshlets.meshlets[meshlet_index], meshes.meshes[mesh_index], cam_pos);
    if (!is_visible) {
        commands.commands[meshlet_index].vertex_count = 0u;
        commands.commands[meshlet_index].instance_count = 0u;
    }
}