{"spirv_code":[],"wgsl_code":"\nstruct ConstantData {\n    view: mat4x4<f32>,\n    proj: mat4x4<f32>,\n    screen_width: f32,\n    screen_height: f32,\n    flags: u32,\n};\n\n\nstruct DrawCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_index: u32,\n    vertex_offset: i32,\n    base_instance: u32,\n};\n\nstruct MeshData {\n    position: vec3<f32>,\n    scale: f32,\n    orientation: vec4<f32>,\n};\n\nstruct MeshletData {\n    center: vec3<f32>,\n    radius: f32,\n    cone_axis: vec3<f32>,\n    cone_cutoff: f32,\n    vertices_count: u32,\n    vertices_offset: u32,\n    indices_count: u32,\n    indices_offset: u32,\n};\n\nstruct Meshlets {\n    meshlets: array<MeshletData>,\n};\nstruct Meshes {\n    meshes: array<MeshData>,\n};\nstruct Commands {\n    commands: array<DrawCommand>,\n};\n\n@group(0) @binding(0)\nvar<uniform> constant_data: ConstantData;\n@group(0) @binding(1)\nvar<storage, read> meshlets: Meshlets;\n@group(0) @binding(2)\nvar<storage, read> meshes: Meshes;\n@group(0) @binding(3)\nvar<storage, read_write> commands: Commands;\n\n\nfn rotate_quat(pos: vec3<f32>, orientation: vec4<f32>) -> vec3<f32> {\n    return pos + 2.0 * cross(orientation.xyz, cross(orientation.xyz, pos) + orientation.w * pos);\n}\n\nfn is_cone_culled(meshlet: MeshletData, mesh: MeshData, camera_position: vec3<f32>) -> bool {\n    let center = rotate_quat(meshlet.center, mesh.orientation) * mesh.scale + mesh.position;\n    let radius = meshlet.radius * mesh.scale;\n\n    let cone_axis = rotate_quat(vec3<f32>(meshlet.cone_axis[0] / 127., meshlet.cone_axis[1] / 127., meshlet.cone_axis[2] / 127.), mesh.orientation);\n    let cone_cutoff = meshlet.cone_cutoff / 127.;\n\n    let direction = center - camera_position;\n    return dot(direction, cone_axis) < cone_cutoff * length(direction) + radius;\n//    let direction = meshlet.center - camera_position;\n//    return dot(direction, meshlet.cone_axis) < meshlet.cone_cutoff * length(direction) + meshlet.radius;\n}\n\n\n@compute\n@workgroup_size(32, 1, 1)\nfn main(@builtin(local_invocation_id) local_invocation_id: vec3<u32>, @builtin(local_invocation_index) local_invocation_index: u32, @builtin(global_invocation_id) global_invocation_id: vec3<u32>, @builtin(workgroup_id) workgroup_id: vec3<u32>) {\n    let total = arrayLength(&meshlets.meshlets);\n    let meshlet_index = global_invocation_id.x;\n    if (meshlet_index >= total) {\n        return;\n    }\n    let mesh_index = commands.commands[meshlet_index].base_instance;\n\n    let is_visible = is_cone_culled(meshlets.meshlets[meshlet_index], meshes.meshes[mesh_index], constant_data.view[3].xyz);\n    if (!is_visible) {\n        commands.commands[meshlet_index].vertex_count = 0u;\n        commands.commands[meshlet_index].instance_count = 0u;\n        commands.commands[meshlet_index].base_index = 0u;\n        commands.commands[meshlet_index].vertex_offset = 0;\n        commands.commands[meshlet_index].base_instance = 0u;\n    }\n}\n"}