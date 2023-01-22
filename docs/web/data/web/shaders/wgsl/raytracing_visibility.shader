{"spirv_code":[],"wgsl_code":"\nfn quantize_unorm(v: f32, n: u32) -> u32 {\n    let scale = f32((1 << n) - 1);\n    return u32(0.5 + (v * scale));\n}\nfn quantize_snorm(v: f32, n: u32) -> u32 {\n    let c = (1u << (n - 1u)) - 1u;\n    let scale = f32(c);\n    if v < 0. {\n        return (u32(-v * scale) & c) | (1u << (n - 1u));\n    } else {\n        return u32(v * scale) & c;\n    }\n}\n\nfn decode_unorm(i: u32, n: u32) -> f32 {    \n    let scale = f32((1 << n) - 1);\n    if (i == 0u) {\n        return 0.;\n    } else if (i == u32(scale)) {\n        return 1.;\n    } else {\n        return (f32(i) - 0.5) / scale;\n    }\n}\n\nfn decode_snorm(i: u32, n: u32) -> f32 {\n    let s = i >> (n - 1u);\n    let c = (1u << (n - 1u)) - 1u;\n    let scale = f32(c);\n    if s > 0u {\n        let r = f32(i & c) / scale;\n        return -r;\n    } else {\n        return f32(i & c) / scale;\n    }\n}\n\n\nfn decode_uv(v: u32) -> vec2<f32> {\n    return unpack2x16float(v);\n}\nfn decode_as_vec3(v: u32) -> vec3<f32> {\n    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_unorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn pack_normal(normal: vec3<f32>) -> vec2<f32> {\n    return vec2<f32>(normal.xy * 0.5 + 0.5);\n}\nfn unpack_normal(uv: vec2<f32>) -> vec3<f32> {\n    return vec3<f32>(uv.xy * 2. - 1., sqrt(1.-dot(uv.xy, uv.xy)));\n}\n\nfn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {\n    let r = quantize_unorm(value.x, 8u) << 24u;\n    let g = quantize_unorm(value.y, 8u) << 16u;\n    let b = quantize_unorm(value.z, 8u) << 8u;\n    let a = quantize_unorm(value.w, 8u);\n    return (r | g | b | a);\n}\nfn unpack_snorm_to_4_f32(v: u32) -> vec4<f32> {\n    let r = decode_snorm((v >> 24u) & 255u, 8u);\n    let g = decode_snorm((v >> 16u) & 255u, 8u);\n    let b = decode_snorm((v >> 8u) & 255u, 8u);\n    let a = decode_snorm(v & 255u, 8u);\n    return vec4<f32>(r,g,b,a);\n}\nfn unpack_unorm_to_4_f32(v: u32) -> vec4<f32> {\n    let r = decode_unorm((v >> 24u) & 255u, 8u);\n    let g = decode_unorm((v >> 16u) & 255u, 8u);\n    let b = decode_unorm((v >> 8u) & 255u, 8u);\n    let a = decode_unorm(v & 255u, 8u);\n    return vec4<f32>(r,g,b,a);\n}\n\n// 0-1 from 0-255\nfn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {\n    let cutoff = srgb < vec3<f32>(10.31475);\n    let lower = srgb / vec3<f32>(3294.6);\n    let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));\n    return select(higher, lower, cutoff);\n}\n\n// [u8; 4] SRGB as u32 -> [r, g, b, a]\nfn unpack_color(color: u32) -> vec4<f32> {\n    return vec4<f32>(\n        f32(color & 255u),\n        f32((color >> 8u) & 255u),\n        f32((color >> 16u) & 255u),\n        f32((color >> 24u) & 255u),\n    );\n}\n\n// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.\nfn hash( x: u32 ) -> u32 {\n    var v = x;\n    v += ( v << 10u );\n    v ^= ( v >>  6u );\n    v += ( v <<  3u );\n    v ^= ( v >> 11u );\n    v += ( v << 15u );\n    return v;\n}\n\n\nfn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) \n{\n    let c = *ptr_a;\n    *ptr_a = *ptr_b;\n    *ptr_b = c;\n}\nconst MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;\nconst MAX_TEXTURE_COORDS_SET: u32 = 4u;\n\nconst TEXTURE_TYPE_BASE_COLOR: u32 = 0u;\nconst TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;\nconst TEXTURE_TYPE_NORMAL: u32 = 2u;\nconst TEXTURE_TYPE_EMISSIVE: u32 = 3u;\nconst TEXTURE_TYPE_OCCLUSION: u32 = 4u;\nconst TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;\nconst TEXTURE_TYPE_DIFFUSE: u32 = 6u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;\nconst TEXTURE_TYPE_COUNT: u32 = 8u;\n\nconst MATERIAL_ALPHA_BLEND_OPAQUE = 0u;\nconst MATERIAL_ALPHA_BLEND_MASK = 1u;\nconst MATERIAL_ALPHA_BLEND_BLEND = 2u;\n\nconst MESH_FLAGS_NONE: u32 = 0u;\nconst MESH_FLAGS_VISIBLE: u32 = 1u;\nconst MESH_FLAGS_OPAQUE: u32 = 2u; // 1 << 1\nconst MESH_FLAGS_TRANSPARENT: u32 = 4u;  // 1 << 2\nconst MESH_FLAGS_WIREFRAME: u32 = 8u; // 1 << 3\nconst MESH_FLAGS_DEBUG: u32 = 16u; // 1 << 4\nconst MESH_FLAGS_UI: u32 = 32u; // 1 << 5\n\nconst CONSTANT_DATA_FLAGS_NONE: u32 = 0u;\nconst CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 2u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE: u32 = 4u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 8u;\n\n\nstruct ConstantData {\n    view: mat4x4<f32>,\n    proj: mat4x4<f32>,\n    inverse_view_proj: mat4x4<f32>,\n    screen_width: f32,\n    screen_height: f32,\n    cam_fov: f32,\n    flags: u32,\n};\n\nstruct Vertex {\n    @location(0) position_and_color_offset: u32,\n    @location(1) normal_offset: i32,\n    @location(2) tangent_offset: i32,\n    @location(3) mesh_index: u32,\n    @location(4) uvs_offset: vec4<i32>,\n};\n\nstruct DrawCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_vertex: u32,\n    base_instance: u32,\n};\n\nstruct DrawIndexedCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_index: u32,\n    vertex_offset: i32,\n    base_instance: u32,\n};\n\nstruct Mesh {\n    vertex_offset: u32,\n    indices_offset: u32,\n    material_index: i32,\n    bhv_index: u32,\n    position: vec3<f32>,\n    meshlets_offset: u32,\n    scale: vec3<f32>,\n    meshlets_count: u32,\n    orientation: vec4<f32>,\n};\n\nstruct ConeCulling {\n    center: vec3<f32>,\n    cone_axis_cutoff: u32,\n};\n\nstruct Meshlet {\n    @location(5) mesh_index: u32,\n    @location(6) indices_offset: u32,\n    @location(7) indices_count: u32,\n    @location(8) bhv_index: u32,\n};\n\nstruct BHVNode {\n    min: vec3<f32>,\n    miss: i32,\n    max: vec3<f32>,\n    reference: i32, //-1 or mesh_index or meshlet_index or triangle_index\n};\n\n\nstruct LightData {\n    position: vec3<f32>,\n    light_type: u32,\n    color: vec4<f32>,\n    intensity: f32,\n    range: f32,\n    inner_cone_angle: f32,\n    outer_cone_angle: f32,\n};\n\nstruct TextureData {\n    texture_index: u32,\n    layer_index: u32,\n    total_width: f32,\n    total_height: f32,\n    area: vec4<f32>,\n};\n\nstruct Material {\n    textures_indices: array<i32, 8>,//TEXTURE_TYPE_COUNT>,\n    textures_coord_set: array<u32, 8>,//TEXTURE_TYPE_COUNT>,\n    roughness_factor: f32,\n    metallic_factor: f32,\n    alpha_cutoff: f32,\n    alpha_mode: u32,\n    base_color: vec4<f32>,\n    emissive_color: vec3<f32>,\n    occlusion_strength: f32,\n    diffuse_color: vec4<f32>,\n    specular_color: vec4<f32>,\n};\n\n\nstruct Lights {\n    data: array<LightData>,\n};\n\nstruct Textures {\n    data: array<TextureData>,\n};\n\nstruct Materials {\n    data: array<Material>,\n};\n\nstruct DrawCommands {\n    data: array<DrawCommand>,\n};\n\nstruct DrawIndexedCommands {\n    data: array<DrawIndexedCommand>,\n};\n\nstruct Meshes {\n    data: array<Mesh>,\n};\n\nstruct Meshlets {\n    data: array<Meshlet>,\n};\n\nstruct Indices {\n    data: array<u32>,\n};\n\nstruct Vertices {\n    data: array<Vertex>,\n};\n\nstruct Matrices {\n    data: array<mat4x4<f32>>,\n};\n\nstruct Positions {\n    data: array<u32>,\n};\n\nstruct Colors {\n    data: array<u32>,\n};\n\nstruct Normals {\n    data: array<u32>,\n};\n\nstruct Tangents {\n    data: array<vec4<f32>>,\n};\n\nstruct UVs {\n    data: array<u32>,\n};\n\nstruct MeshletsCulling {\n    data: array<ConeCulling>,\n};\n\nstruct BHV {\n    data: array<BHVNode>,\n};\n\nstruct MeshFlags {\n    data: array<u32>,\n};\n\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) uv: vec2<f32>,\n};\n\nstruct FragmentOutput {\n    @location(0) color: vec4<f32>,\n};\n\n@group(0) @binding(0)\nvar<uniform> constant_data: ConstantData;\n@group(0) @binding(1)\nvar<storage, read> indices: Indices;\n@group(0) @binding(2)\nvar<storage, read> vertices: Vertices;\n@group(0) @binding(3)\nvar<storage, read> positions: Positions;\n@group(0) @binding(4)\nvar<storage, read> meshes: Meshes;\n@group(0) @binding(5)\nvar<storage, read> meshlets: Meshlets;\n@group(0) @binding(6)\nvar<storage, read> meshlets_culling: MeshletsCulling;\n\n@group(1) @binding(0)\nvar<storage, read> tlas: BHV;\n@group(1) @binding(1)\nvar<storage, read> bhv: BHV;\n@group(1) @binding(2)\nvar<storage, read> meshes_inverse_matrix: Matrices;\n\n@group(2) @binding(0)\nvar render_target: texture_storage_2d<rgba8unorm, read_write>;\n\n\nfn extract_scale(m: mat4x4<f32>) -> vec3<f32> \n{\n    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);\n    let sx = length(s[0]);\n    let sy = length(s[1]);\n    let det = determinant(s);\n    var sz = length(s[2]);\n    if (det < 0.) \n    {\n        sz = -sz;\n    }\n    return vec3<f32>(sx, sy, sz);\n}\n\nfn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> \n{\n    if (row == 1u) {\n        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);\n    } else if (row == 2u) {\n        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);\n    } else if (row == 3u) {\n        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);\n    } else {        \n        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);\n    }\n}\n\nfn normalize_plane(plane: vec4<f32>) -> vec4<f32> \n{\n    return (plane / length(plane.xyz));\n}\n\nfn unproject(ncd_pos: vec2<f32>, depth: f32) -> vec3<f32> \n{    \n    var world_pos = constant_data.inverse_view_proj * vec4<f32>(ncd_pos, depth, 1. );\n    world_pos /= world_pos.w;\n    return world_pos.xyz;\n}\n\nfn rotate_vector(v: vec3<f32>, orientation: vec4<f32>) -> vec3<f32> \n{\n    return v + 2. * cross(orientation.xyz, cross(orientation.xyz, v) + orientation.w * v);\n}\n\nfn transform_vector(v: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> vec3<f32> \n{\n    return rotate_vector(v, orientation) * scale + position;\n}\nconst MAX_FLOAT: f32 = 3.402823466e+38;\nconst HIT_EPSILON: f32 = 0.0001;\nconst INVALID_NODE: i32 = -1;\nconst MAX_STEPS: u32 = 1024u;\n\nstruct Ray {\n    origin: vec3<f32>,\n    direction: vec3<f32>,\n}\n\nstruct Result {\n    distance: f32,\n    visibility_id: u32,\n}\n\nfn compute_ray(image_pixel: vec2<u32>, image_size: vec2<u32>) -> Ray {\n    var clip_coords = 2. * (vec2<f32>(image_pixel) / vec2<f32>(image_size)) - vec2<f32>(1., 1.);\n    clip_coords.y = -clip_coords.y;\n    \n    let origin = unproject(clip_coords.xy, -1.);\n    let far = unproject(clip_coords.xy, 1.);\n    let direction = normalize(far - origin);\n    \n    let ray = Ray(origin, direction);\n    return ray;\n}\n\nfn intersect_aabb(r: Ray, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 \n{     \n    let inverse_dir = 1. / r.direction;\n    let v_min = (aabb_min - r.origin) * inverse_dir;\n    let v_max = (aabb_max - r.origin) * inverse_dir;\n\n    let t_min = min(v_min, v_max);\n    let t_max = max(v_min, v_max);\n\n    let t_near = max(max(t_min.x, t_min.y), max(t_min.x, t_min.z));\n    let t_far = min(min(t_max.x, t_max.y), min(t_max.x, t_max.z));\n\n    if (t_near > t_far || t_far < 0.) \n    { \n        return MAX_FLOAT; \n    }\n    if (t_near > 0.) {\n        return t_near;\n    } else {\n        return t_far;\n    } \n}\n\n\nfn intersect_triangle(r: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32\n{\n    let e1 = v1 - v0;\n    let e2 = v2 - v0;\n\n    let p = cross(r.direction, e2);\n    let det = dot(e1, p);\n    \n    if (abs(det) < HIT_EPSILON) { return MAX_FLOAT; }\n\n    // Computes Barycentric coordinates.\n    let inv_det = 1. / det;\n    let t1 = r.origin - v0;    \n    let u = dot(t1, p) * inv_det;\n    if (u < 0. || u > 1.) { return MAX_FLOAT; }\n    \n    let q = cross(t1, e1);\n    let v = dot(r.direction, q) * inv_det;\n    if (v < 0. || u + v > 1.) { return MAX_FLOAT; }\n\n    let t2 = dot(e2, q) * inv_det;\n    if (t2 < 0.) { return MAX_FLOAT; }\n    return t2;\n}\n\nfn intersect_meshlet_primitive(r: Ray, mesh_id: u32, meshlet_id: u32, primitive_id: u32) -> Result {\n    let mesh = &meshes.data[mesh_id];    \n    let node = &bhv.data[u32((*mesh).bhv_index)];  \n    var nearest = MAX_FLOAT;  \n    var visibility_id = 0u;\n    \n    let meshlet = &meshlets.data[meshlet_id];\n    \n    let index_offset = (*mesh).indices_offset + (*meshlet).indices_offset + (primitive_id * 3u);\n    let i1 = indices.data[index_offset];\n    let i2 = indices.data[index_offset + 1u];\n    let i3 = indices.data[index_offset + 2u];\n\n    let v1 = &vertices.data[(*mesh).vertex_offset + i1];\n    let v2 = &vertices.data[(*mesh).vertex_offset + i2];\n    let v3 = &vertices.data[(*mesh).vertex_offset + i3];\n    \n    let oobb_size = (*node).max - (*node).min;\n    \n    let p1 = (*node).min + decode_as_vec3(positions.data[(*v1).position_and_color_offset]) * oobb_size;\n    let p2 = (*node).min + decode_as_vec3(positions.data[(*v2).position_and_color_offset]) * oobb_size;\n    let p3 = (*node).min + decode_as_vec3(positions.data[(*v3).position_and_color_offset]) * oobb_size;\n    \n    let hit_distance = intersect_triangle(r, p1, p2, p3);\n    if (hit_distance < nearest) {\n        visibility_id = (meshlet_id + 1u) << 8u | primitive_id;\n        nearest = hit_distance;\n    }\n    return Result(nearest, visibility_id);\n}\n\nfn is_meshlet_frontfacing(r: Ray, mesh_id: u32, meshlet_id: u32) -> bool \n{    \n    let mesh = &meshes.data[mesh_id];   \n    let meshlet = &meshlets.data[meshlet_id];\n    let cone_culling = &meshlets_culling.data[meshlet_id];\n    let cone_axis_cutoff = unpack4x8snorm((*cone_culling).cone_axis_cutoff);\n\n    let direction = (*cone_culling).center - r.origin;\n    let is_front_facing = dot(normalize(direction), cone_axis_cutoff.xyz) < (cone_axis_cutoff.w * length(direction));\n    return is_front_facing;\n}\n\nfn intersect_meshlet(r: Ray, mesh_id: u32, meshlet_id: u32) -> Result {\n    var nearest = MAX_FLOAT;  \n    var visibility_id = 0u;\n    if (!is_meshlet_frontfacing(r, mesh_id, meshlet_id)) {\n        return Result(nearest, visibility_id);\n    }\n    \n    let mesh = &meshes.data[mesh_id];   \n    let meshlet = &meshlets.data[meshlet_id];\n    let node = &bhv.data[u32((*mesh).bhv_index)];  \n    let primitive_count = (*meshlet).indices_count / 3u;\n    for(var primitive_id = 0u; primitive_id < primitive_count; primitive_id = primitive_id + 1u)\n    {       \n        let hit = intersect_meshlet_primitive(r, mesh_id, meshlet_id, primitive_id);\n        if (hit.distance < nearest) {\n            visibility_id = hit.visibility_id;\n            nearest = hit.distance;\n        }\n    }\n    return Result(nearest, visibility_id);\n}\n\nfn traverse_meshlets(r: Ray, mesh_id: u32) -> Result {\n    let mesh = &meshes.data[mesh_id];    \n    var nearest = MAX_FLOAT;  \n    var visibility_id = 0u;\n    for(var i = 0u; i < (*mesh).meshlets_count; i = i + 1u)\n    {\n        let meshlet_id = (*mesh).meshlets_offset + i;\n        let hit = intersect_meshlet(r, mesh_id, meshlet_id);\n        if (hit.distance < nearest) {\n            visibility_id = hit.visibility_id;\n            nearest = hit.distance;\n        }\n    }\n    return Result(nearest, visibility_id);\n}\n\nfn traverse_forced_all_bhv_nodes(r: Ray, mesh_id: u32, bhv_start: i32, bhv_end: i32) -> Result {\n    let mesh = &meshes.data[mesh_id];    \n    let mesh_bhv_index = i32((*mesh).bhv_index);\n    var bhv_index = bhv_start;\n    let nodes_count = i32(arrayLength(&bhv.data));\n    var nearest = MAX_FLOAT;  \n    var visibility_id = 0u;\n\n    var steps = 0u;\n    while (bhv_index >= bhv_start && bhv_index < bhv_end && bhv_index < nodes_count)\n    { \n        let node = &bhv.data[u32(bhv_index)];   \n        let intersection = intersect_aabb(r, (*node).min, (*node).max);\n        if (intersection < nearest) {\n            if ((*node).reference >= 0) {\n                //leaf node\n                let meshlet_id = (*mesh).meshlets_offset + u32((*node).reference);\n                let hit = intersect_meshlet(r, mesh_id, meshlet_id);\n                if (hit.distance < nearest) {\n                    visibility_id = hit.visibility_id;\n                    nearest = hit.distance;\n                }\n            }\n        }       \n        bhv_index = bhv_index + 1;\n    }\n    return Result(nearest, visibility_id);\n}\n\nfn traverse_bhv_of_meshlets(r: Ray, mesh_id: u32) -> Result {\n    let mesh = &meshes.data[mesh_id];    \n    var bhv_index = i32((*mesh).bhv_index);    \n    let mesh_bhv_index = bhv_index;\n    var nearest = MAX_FLOAT;  \n    var visibility_id = 0u;\n\n    var steps = 0u;\n    while (bhv_index >= 0 && steps < MAX_STEPS)\n    { \n        let node = &bhv.data[u32(bhv_index)];   \n        let intersection = intersect_aabb(r, (*node).min, (*node).max);\n        if (intersection < nearest) {\n            if ((*node).reference < 0) {\n                //inner node\n                bhv_index = bhv_index + 1;\n                steps = steps + 1u;\n                continue;  \n            }\n            //leaf node\n            let meshlet_id = (*mesh).meshlets_offset + u32((*node).reference);\n            let hit = intersect_meshlet(r, mesh_id, meshlet_id);\n            if (hit.distance < nearest) {\n                visibility_id = hit.visibility_id;\n                nearest = hit.distance;\n            }         \n        }        \n        bhv_index = (*node).miss;\n        if bhv_index >= 0 {\n            bhv_index += mesh_bhv_index;\n        }\n        steps = steps + 1u;\n    }\n    return Result(nearest, visibility_id);\n}\n\n\n\nfn traverse_bhv_of_meshlets_and_primitives(r: Ray, mesh_id: u32) -> Result {\n    let mesh = &meshes.data[mesh_id];    \n    var bhv_index = i32((*mesh).bhv_index);    \n    let mesh_bhv_index = bhv_index;\n    var nearest = MAX_FLOAT;  \n    var visibility_id = 0u;\n\n    var steps = 0u;\n    while (bhv_index >= 0 && steps < MAX_STEPS)\n    { \n        let node = &bhv.data[u32(bhv_index)];   \n        let intersection = intersect_aabb(r, (*node).min, (*node).max);\n        if (intersection < nearest) {\n            if ((*node).reference >= 0) {\n                //leaf node\n                let meshlet_id = (*mesh).meshlets_offset + (u32((*node).reference) >> 8u) - 1u; \n                let primitive_id = u32((*node).reference) & 255u;\n                let hit = intersect_meshlet_primitive(r, mesh_id, meshlet_id, primitive_id);\n                if (hit.distance < nearest) {\n                    visibility_id = hit.visibility_id;\n                    nearest = hit.distance;\n                }\n            } else {\n                //inner node\n                bhv_index = bhv_index + 1;\n                steps = steps + 1u;\n                continue;                \n            }\n        }        \n        bhv_index = (*node).miss;\n        if bhv_index >= 0 {\n            bhv_index += mesh_bhv_index;\n        }\n        steps = steps + 1u;\n    }\n    return Result(nearest, visibility_id);\n}\n\n\n@compute\n@workgroup_size(8, 8, 1)\nfn main(\n    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, \n    @builtin(local_invocation_index) local_invocation_index: u32, \n    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, \n    @builtin(workgroup_id) workgroup_id: vec3<u32>\n) {\n    let dimensions = vec2<u32>(textureDimensions(render_target));\n\n    let pixel = vec2<u32>(workgroup_id.x * 8u + local_invocation_id.x, \n                          workgroup_id.y * 8u + local_invocation_id.y);\n    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y)\n    {\n        return;\n    }    \n    // Create a ray with the current fragment as the origin.\n    let ray = compute_ray(pixel, dimensions);\n    var nearest = MAX_FLOAT;  \n    var visibility_id = 0u;\n    \n    var tlas_index = 0;\n    \n    while (tlas_index >= 0)\n    {\n        let node = &tlas.data[u32(tlas_index)];    \n        let intersection = intersect_aabb(ray, (*node).min, (*node).max);\n        if (intersection < nearest) {\n            if ((*node).reference >= 0) {\n                //leaf node\n                let mesh_id = u32((*node).reference);\n                let inverse_matrix = &meshes_inverse_matrix.data[mesh_id];    \n                let transformed_origin = (*inverse_matrix) * vec4<f32>(ray.origin, 1.);\n                let transformed_direction = (*inverse_matrix) * vec4<f32>(ray.direction, 0.);\n                let transformed_ray = Ray(transformed_origin.xyz, transformed_direction.xyz);\n                let result = traverse_bhv_of_meshlets(transformed_ray, mesh_id);\n                if (result.visibility_id > 0u && result.distance < nearest) {\n                    visibility_id = result.visibility_id;\n                    nearest = result.distance;\n                }\n            } else {\n                //inner node\n                tlas_index = tlas_index + 1;\n                continue;\n            }\n        }\n        tlas_index = (*node).miss;\n    } \n    \n    textureStore(render_target, vec2<i32>(pixel), unpack4x8unorm(visibility_id));\n}\n"}