{"spirv_code":[],"wgsl_code":"fn decode_unorm(i: u32, n: u32) -> f32 {    \n    let scale = f32((1 << n) - 1);\n    if i == 0u {\n        return 0.;\n    } else if i == u32(scale) {\n        return 1.;\n    } else {\n        return (f32(i) - 0.5) / scale;\n    }\n}\n\nfn decode_snorm(i: i32, n: u32) -> f32 {\n    let scale = f32(1 << (n - 1u));\n    return (f32(i) / scale);\n}\n\n\nfn decode_uv(v: u32) -> vec2<f32> {\n    return unpack2x16float(v);\n}\nfn decode_as_vec3(v: u32) -> vec3<f32> {\n    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_unorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn pack_normal(normal: vec3<f32>) -> vec2<f32> {\n    return vec2<f32>(normal.xy * 0.5 + 0.5);\n}\nfn unpack_normal(uv: vec2<f32>) -> vec3<f32> {\n    return vec3<f32>(uv.xy * 2. - 1., sqrt(1.-dot(uv.xy, uv.xy)));\n}\n\nfn quantize_unorm(v: f32, n: u32) -> u32 {\n    let scale = f32((1 << n) - 1);\n    return u32(0.5 + (v * scale));\n}\n\nfn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {\n    let r = quantize_unorm(value.x, 8u) << 24u;\n    let g = quantize_unorm(value.y, 8u) << 16u;\n    let b = quantize_unorm(value.z, 8u) << 8u;\n    let a = quantize_unorm(value.w, 8u);\n    return (r | g | b | a);\n}\nfn unpack_unorm_to_4_f32(color: u32) -> vec4<f32> {\n    return vec4<f32>(\n        f32((color >> 24u) & 255u),\n        f32((color >> 16u) & 255u),\n        f32((color >> 8u) & 255u),\n        f32(color & 255u),\n    );\n}\n\nfn hash(index: u32) -> u32 {\n    var v = index;\n    v = (v + 0x7ed55d16u) + (v << 12u);\n    v = (v ^ 0xc761c23cu) ^ (v >> 19u);\n    v = (v + 0x165667b1u) + (v << 5u);\n    v = (v + 0xd3a2646cu) ^ (v << 9u);\n    v = (v + 0xfd7046c5u) + (v << 3u);\n    v = (v ^ 0xb55a4f09u) ^ (v >> 16u);\n    return v;\n}\n\n// 0-1 from 0-255\nfn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {\n    let cutoff = srgb < vec3<f32>(10.31475);\n    let lower = srgb / vec3<f32>(3294.6);\n    let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));\n    return select(higher, lower, cutoff);\n}\n\n// [u8; 4] SRGB as u32 -> [r, g, b, a]\nfn unpack_color(color: u32) -> vec4<f32> {\n    return vec4<f32>(\n        f32(color & 255u),\n        f32((color >> 8u) & 255u),\n        f32((color >> 16u) & 255u),\n        f32((color >> 24u) & 255u),\n    );\n}\n\nfn extract_scale(m: mat4x4<f32>) -> vec3<f32> {\n    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);\n    let sx = length(s[0]);\n    let sy = length(s[1]);\n    let det = determinant(s);\n    var sz = length(s[2]);\n    if (det < 0.) {\n        sz = -sz;\n    }\n    return vec3<f32>(sx, sy, sz);\n}\n\nfn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> {\n    if (row == 1u) {\n        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);\n    } else if (row == 2u) {\n        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);\n    } else if (row == 3u) {\n        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);\n    } else {        \n        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);\n    }\n}\n\nfn normalize_plane(plane: vec4<f32>) -> vec4<f32> {\n    return (plane / length(plane.xyz));\n}\nlet MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;\nlet MAX_TEXTURE_COORDS_SET: u32 = 4u;\n\nlet TEXTURE_TYPE_BASE_COLOR: u32 = 0u;\nlet TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;\nlet TEXTURE_TYPE_NORMAL: u32 = 2u;\nlet TEXTURE_TYPE_EMISSIVE: u32 = 3u;\nlet TEXTURE_TYPE_OCCLUSION: u32 = 4u;\nlet TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;\nlet TEXTURE_TYPE_DIFFUSE: u32 = 6u;\nlet TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;\nlet TEXTURE_TYPE_COUNT: u32 = 8u;\n\nlet MATERIAL_ALPHA_BLEND_OPAQUE = 0u;\nlet MATERIAL_ALPHA_BLEND_MASK = 1u;\nlet MATERIAL_ALPHA_BLEND_BLEND = 2u;\n\nlet MESH_FLAGS_NONE: u32 = 0u;\nlet MESH_FLAGS_VISIBLE: u32 = 1u;\nlet MESH_FLAGS_OPAQUE: u32 = 2u; // 1 << 1\nlet MESH_FLAGS_TRANSPARENT: u32 = 4u;  // 1 << 2\nlet MESH_FLAGS_WIREFRAME: u32 = 8u; // 1 << 3\nlet MESH_FLAGS_DEBUG: u32 = 16u; // 1 << 4\nlet MESH_FLAGS_UI: u32 = 32u; // 1 << 5\n\nlet CONSTANT_DATA_FLAGS_NONE: u32 = 0u;\nlet CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 2u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE: u32 = 4u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 8u;\n\n\nstruct ConstantData {\n    view: mat4x4<f32>,\n    proj: mat4x4<f32>,\n    inverse_view_proj: mat4x4<f32>,\n    screen_width: f32,\n    screen_height: f32,\n    flags: u32,\n};\n\nstruct DrawVertex {\n    @location(0) position_and_color_offset: u32,\n    @location(1) normal_offset: i32,\n    @location(2) tangent_offset: i32,\n    @location(3) mesh_index: u32,\n    @location(4) uvs_offset: vec4<i32>,\n};\n\nstruct DrawCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_vertex: u32,\n    base_instance: u32,\n};\n\nstruct DrawIndexedCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_index: u32,\n    vertex_offset: i32,\n    base_instance: u32,\n};\n\nstruct DrawMesh {\n    vertex_offset: u32,\n    indices_offset: u32,\n    material_index: i32,\n    mesh_flags: u32,\n    aabb_min: vec3<f32>,\n    meshlet_offset: u32,\n    aabb_max: vec3<f32>,\n    meshlet_count: u32,\n    transform: mat4x4<f32>,\n};\n\nstruct DrawMeshlet {\n    @location(5) mesh_index: u32,\n    @location(6) vertex_offset: u32,\n    @location(7) indices_offset: u32,\n    @location(8) indices_count: u32,\n    @location(9) center_radius: vec4<f32>,\n    @location(10) cone_axis_cutoff: vec4<f32>,\n};\n\n\nstruct LightData {\n    position: vec3<f32>,\n    light_type: u32,\n    color: vec4<f32>,\n    intensity: f32,\n    range: f32,\n    inner_cone_angle: f32,\n    outer_cone_angle: f32,\n};\n\nstruct TextureData {\n    texture_index: u32,\n    layer_index: u32,\n    total_width: f32,\n    total_height: f32,\n    area: vec4<f32>,\n};\n\nstruct DrawMaterial {\n    textures_indices: array<i32, 8>,//TEXTURE_TYPE_COUNT>,\n    textures_coord_set: array<u32, 8>,//TEXTURE_TYPE_COUNT>,\n    roughness_factor: f32,\n    metallic_factor: f32,\n    alpha_cutoff: f32,\n    alpha_mode: u32,\n    base_color: vec4<f32>,\n    emissive_color: vec3<f32>,\n    occlusion_strength: f32,\n    diffuse_color: vec4<f32>,\n    specular_color: vec4<f32>,\n};\n\n\nstruct Lights {\n    data: array<LightData>,\n};\n\nstruct Textures {\n    data: array<TextureData>,\n};\n\nstruct Materials {\n    data: array<DrawMaterial>,\n};\n\nstruct DrawCommands {\n    data: array<DrawCommand>,\n};\n\nstruct RenderCommands {\n    data: array<DrawIndexedCommand>,\n};\n\nstruct Meshes {\n    data: array<DrawMesh>,\n};\n\nstruct Meshlets {\n    data: array<DrawMeshlet>,\n};\n\nstruct Indices {\n    data: array<u32>,\n};\n\nstruct Vertices {\n    data: array<DrawVertex>,\n};\n\nstruct Matrices {\n    data: array<mat4x4<f32>>,\n};\n\nstruct Positions {\n    data: array<u32>,\n};\n\nstruct Colors {\n    data: array<u32>,\n};\n\nstruct Normals {\n    data: array<u32>,\n};\n\nstruct Tangents {\n    data: array<vec4<f32>>,\n};\n\nstruct UVs {\n    data: array<u32>,\n};\n\n\n\nstruct VisibleInstances {\n    data: array<u32>,\n};\n\n@group(0) @binding(0)\nvar<uniform> constant_data: ConstantData;\n@group(0) @binding(1)\nvar<storage, read> meshlets: Meshlets;\n@group(0) @binding(2)\nvar<storage, read> meshes: Meshes;\n@group(0) @binding(3)\nvar<storage, read_write> count: atomic<u32>;\n@group(0) @binding(4)\nvar<storage, read_write> commands: RenderCommands;\n\n\nfn transform_vector(v: vec3<f32>, q: vec4<f32>) -> vec3<f32> {\n    return v + 2. * cross(q.xyz, cross(q.xyz, v) + q.w * v);\n}\n\n//ScreenSpace Frustum Culling\nfn is_inside_frustum(center: vec3<f32>, radius: f32) -> bool {\n    let mvp = constant_data.proj * constant_data.view;\n    let row0 = matrix_row(mvp, 0u);\n    let row1 = matrix_row(mvp, 1u);\n    let row3 = matrix_row(mvp, 3u);\n    let frustum_1 = normalize_plane(row3 + row0);\n    let frustum_2 = normalize_plane(row3 - row0);\n    let frustum_3 = normalize_plane(row3 + row1);\n    let frustum_4 = normalize_plane(row3 - row1);\n    var visible: bool = true;    \n    visible = visible && (dot(frustum_1.xyz, center) + frustum_1.w + radius > 0.);\n    visible = visible && (dot(frustum_2.xyz, center) + frustum_2.w + radius > 0.);\n    visible = visible && (dot(frustum_3.xyz, center) + frustum_3.w + radius > 0.);\n    visible = visible && (dot(frustum_4.xyz, center) + frustum_4.w + radius > 0.);    \n    return visible;\n}\n\n//fn is_cone_culled(center: vec3<f32>, radius: f32, cone_axis: vec3<f32>, cone_cutoff: f32, orientation: vec4<f32>, camera_position: vec3<f32>) -> bool {\n//    let axis = transform_vector(cone_axis, orientation);\n//\n//    let direction = center - camera_position;\n//    return dot(direction, axis) < cone_cutoff * length(direction) + radius;\n//}\n\n\n@compute\n@workgroup_size(32, 1, 1)\nfn main(\n    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, \n    @builtin(local_invocation_index) local_invocation_index: u32, \n    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, \n    @builtin(workgroup_id) workgroup_id: vec3<u32>\n) {\n    let total = arrayLength(&meshlets.data);\n    let meshlet_id = global_invocation_id.x;\n    if (meshlet_id >= total) {\n        return;\n    }\n    let meshlet = &meshlets.data[meshlet_id];\n    let mesh_id = (*meshlet).mesh_index;\n    let mesh = &meshes.data[mesh_id];\n    let m = &(*mesh).transform;\n\n    let scale = extract_scale((*m));\n    let center = (*m) * vec4<f32>((*meshlet).center_radius.xyz, 1.0);\n    let radius = (*meshlet).center_radius.w * scale.x;\n    let view_pos = constant_data.view[3].xyz;\n\n    if (is_inside_frustum(center.xyz, radius)) \n    {\n        let index = atomicAdd(&count, 1u);\n        let command = &commands.data[index];\n        (*command).vertex_count = (*meshlet).indices_count;\n        (*command).instance_count = 1u;\n        (*command).base_index = (*mesh).indices_offset + (*meshlet).indices_offset;\n        (*command).vertex_offset = i32((*mesh).vertex_offset);\n        (*command).base_instance = meshlet_id;\n    } \n    \n    //let cone_axis = vec3<f32>((*meshlet).cone_axis[0], (*meshlet).cone_axis[1], (*meshlet).cone_axis[2]);\n    //is_visible = is_cone_culled(center.xyz, radius, cone_axis, (*meshlet).cone_cutoff, meshes.meshes[mesh_index].orientation, view_pos);\n    //if (!is_visible) {\n    //    (*command).vertex_count = 0u;\n    //    (*command).instance_count = 0u;\n    //    return;\n    //}\n}\n"}