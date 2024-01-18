{"spirv_code":[],"wgsl_code":"fn quantize_unorm(v: f32, n: u32) -> u32 {\n    let scale = f32((1 << n) - 1);\n    return u32(0.5 + (v * scale));\n}\nfn quantize_snorm(v: f32, n: u32) -> u32 {\n    let c = (1u << (n - 1u)) - 1u;\n    let scale = f32(c);\n    if v < 0. {\n        return (u32(-v * scale) & c) | (1u << (n - 1u));\n    } else {\n        return u32(v * scale) & c;\n    }\n}\n\nfn decode_unorm(i: u32, n: u32) -> f32 {    \n    let scale = f32((1 << n) - 1);\n    if (i == 0u) {\n        return 0.;\n    } else if (i == u32(scale)) {\n        return 1.;\n    } else {\n        return (f32(i) - 0.5) / scale;\n    }\n}\n\nfn decode_snorm(i: u32, n: u32) -> f32 {\n    let s = i >> (n - 1u);\n    let c = (1u << (n - 1u)) - 1u;\n    let scale = f32(c);\n    if s > 0u {\n        let r = f32(i & c) / scale;\n        return -r;\n    } else {\n        return f32(i & c) / scale;\n    }\n}\n\n\nfn decode_uv(v: u32) -> vec2<f32> {\n    return unpack2x16float(v);\n}\nfn decode_as_vec3(v: u32) -> vec3<f32> {\n    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_unorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn pack_normal(normal: vec3<f32>) -> vec2<f32> {\n    return vec2<f32>(normal.xy * 0.5 + 0.5);\n}\nfn unpack_normal(uv: vec2<f32>) -> vec3<f32> {\n    return vec3<f32>(uv.xy * 2. - 1., sqrt(1.-dot(uv.xy, uv.xy)));\n}\n\nfn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {\n    let r = quantize_unorm(value.x, 8u) << 24u;\n    let g = quantize_unorm(value.y, 8u) << 16u;\n    let b = quantize_unorm(value.z, 8u) << 8u;\n    let a = quantize_unorm(value.w, 8u);\n    return (r | g | b | a);\n}\nfn unpack_snorm_to_4_f32(v: u32) -> vec4<f32> {\n    let r = decode_snorm((v >> 24u) & 255u, 8u);\n    let g = decode_snorm((v >> 16u) & 255u, 8u);\n    let b = decode_snorm((v >> 8u) & 255u, 8u);\n    let a = decode_snorm(v & 255u, 8u);\n    return vec4<f32>(r,g,b,a);\n}\nfn unpack_unorm_to_4_f32(v: u32) -> vec4<f32> {\n    let r = decode_unorm((v >> 24u) & 255u, 8u);\n    let g = decode_unorm((v >> 16u) & 255u, 8u);\n    let b = decode_unorm((v >> 8u) & 255u, 8u);\n    let a = decode_unorm(v & 255u, 8u);\n    return vec4<f32>(r,g,b,a);\n}\n\n// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.\nfn hash( x: u32 ) -> u32 {\n    var v = x;\n    v += ( v << 10u );\n    v ^= ( v >>  6u );\n    v += ( v <<  3u );\n    v ^= ( v >> 11u );\n    v += ( v << 15u );\n    return v;\n}\n\n\nfn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) \n{\n    let c = *ptr_a;\n    *ptr_a = *ptr_b;\n    *ptr_b = c;\n}\nconst MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;\nconst MAX_TEXTURE_COORDS_SET: u32 = 4u;\n\nconst TEXTURE_TYPE_BASE_COLOR: u32 = 0u;\nconst TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;\nconst TEXTURE_TYPE_NORMAL: u32 = 2u;\nconst TEXTURE_TYPE_EMISSIVE: u32 = 3u;\nconst TEXTURE_TYPE_OCCLUSION: u32 = 4u;\nconst TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;\nconst TEXTURE_TYPE_DIFFUSE: u32 = 6u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;\nconst TEXTURE_TYPE_COUNT: u32 = 8u;\n\nconst MATERIAL_ALPHA_BLEND_OPAQUE = 0u;\nconst MATERIAL_ALPHA_BLEND_MASK = 1u;\nconst MATERIAL_ALPHA_BLEND_BLEND = 2u;\n\nconst MESH_FLAGS_NONE: u32 = 0u;\nconst MESH_FLAGS_VISIBLE: u32 = 1u;\nconst MESH_FLAGS_OPAQUE: u32 = 2u; // 1 << 1\nconst MESH_FLAGS_TRANSPARENT: u32 = 4u;  // 1 << 2\nconst MESH_FLAGS_WIREFRAME: u32 = 8u; // 1 << 3\nconst MESH_FLAGS_DEBUG: u32 = 16u; // 1 << 4\nconst MESH_FLAGS_UI: u32 = 32u; // 1 << 5\n\nconst CONSTANT_DATA_FLAGS_NONE: u32 = 0u;\nconst CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 2u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE: u32 = 4u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 8u;\n\nconst MAX_FLOAT: f32 = 3.402823466e+38;\n\nconst RAY_STEP_FLAGS_NONE: u32 = 0u;\nconst RAY_STEP_FLAGS_COMPUTE_RAY: u32 = 1u;\nconst RAY_STEP_FLAGS_TLAS: u32 = 2u; // 1 << 1\nconst RAY_STEP_FLAGS_BLAS: u32 = 4u; // 1 << 2\nconst RAY_STEP_FLAGS_MESHLET: u32 = 8u;  // 1 << 3\nconst RAY_STEP_FLAGS_BOUNCE: u32 = 16u; // 1 << 4\n\nstruct RayPayload {\n    origin: vec3<f32>,\n    pixel_x: u32,\n    direction: vec3<f32>,\n    pixel_y: u32,\n};\n\nstruct RayJob {\n    index: u32,\n    step: u32,\n}\n\nstruct ConstantData {\n    view: mat4x4<f32>,\n    proj: mat4x4<f32>,\n    inverse_view_proj: mat4x4<f32>,\n    screen_width: f32,\n    screen_height: f32,\n    cam_fov: f32,\n    flags: u32,\n};\n\nstruct Vertex {\n    @location(0) position_and_color_offset: u32,\n    @location(1) normal_offset: i32,\n    @location(2) tangent_offset: i32,\n    @location(3) mesh_index: u32,\n    @location(4) uvs_offset: vec4<i32>,\n};\n\nstruct DrawCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_vertex: u32,\n    base_instance: u32,\n};\n\nstruct DrawIndexedCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_index: u32,\n    vertex_offset: i32,\n    base_instance: u32,\n};\n\nstruct Mesh {\n    vertex_offset: u32,\n    indices_offset: u32,\n    material_index: i32,\n    bhv_index: u32,\n    position: vec3<f32>,\n    meshlets_offset: u32,\n    scale: vec3<f32>,\n    meshlets_count: u32,\n    orientation: vec4<f32>,\n};\n\nstruct ConeCulling {\n    center: vec3<f32>,\n    cone_axis_cutoff: u32,\n};\n\nstruct Meshlet {\n    @location(5) mesh_index: u32,\n    @location(6) indices_offset: u32,\n    @location(7) indices_count: u32,\n    @location(8) bhv_index: u32,\n};\n\nstruct BHVNode {\n    min: vec3<f32>,\n    miss: i32,\n    max: vec3<f32>,\n    reference: i32, //-1 or mesh_index or meshlet_index or triangle_index\n};\n\n\nstruct LightData {\n    position: vec3<f32>,\n    light_type: u32,\n    color: vec4<f32>,\n    intensity: f32,\n    range: f32,\n    inner_cone_angle: f32,\n    outer_cone_angle: f32,\n};\n\nstruct TextureData {\n    texture_index: u32,\n    layer_index: u32,\n    total_width: f32,\n    total_height: f32,\n    area: vec4<f32>,\n};\n\nstruct Material {\n    textures_indices: array<i32, 8>,//TEXTURE_TYPE_COUNT>,\n    textures_coord_set: array<u32, 8>,//TEXTURE_TYPE_COUNT>,\n    roughness_factor: f32,\n    metallic_factor: f32,\n    alpha_cutoff: f32,\n    alpha_mode: u32,\n    base_color: vec4<f32>,\n    emissive_color: vec3<f32>,\n    occlusion_strength: f32,\n    diffuse_color: vec4<f32>,\n    specular_color: vec4<f32>,\n};\n\n\nstruct Lights {\n    data: array<LightData>,\n};\n\nstruct Textures {\n    data: array<TextureData>,\n};\n\nstruct Materials {\n    data: array<Material>,\n};\n\nstruct DrawCommands {\n    data: array<DrawCommand>,\n};\n\nstruct DrawIndexedCommands {\n    data: array<DrawIndexedCommand>,\n};\n\nstruct Meshes {\n    data: array<Mesh>,\n};\n\nstruct Meshlets {\n    data: array<Meshlet>,\n};\n\nstruct Indices {\n    data: array<u32>,\n};\n\nstruct Vertices {\n    data: array<Vertex>,\n};\n\nstruct Matrices {\n    data: array<mat4x4<f32>>,\n};\n\nstruct Positions {\n    data: array<u32>,\n};\n\nstruct Colors {\n    data: array<u32>,\n};\n\nstruct Normals {\n    data: array<u32>,\n};\n\nstruct Tangents {\n    data: array<vec4<f32>>,\n};\n\nstruct UVs {\n    data: array<u32>,\n};\n\nstruct MeshletsCulling {\n    data: array<ConeCulling>,\n};\n\nstruct BHV {\n    data: array<BHVNode>,\n};\n\nstruct MeshFlags {\n    data: array<u32>,\n};\n\n\nstruct Ray {\n    origin: vec3<f32>,\n    t_min: f32,\n    direction: vec3<f32>,\n    t_max: f32,\n};\n\nstruct Rays {\n    data: array<Ray>,\n};\n\nstruct Data {\n    width: u32,\n    height: u32,\n};\n\n\n@group(0) @binding(0)\nvar<uniform> data: Data;\n@group(0) @binding(1)\nvar<storage, read> indices: Indices;\n@group(0) @binding(2)\nvar<storage, read> vertices: Vertices;\n@group(0) @binding(3)\nvar<storage, read> positions: Positions;\n@group(0) @binding(4)\nvar<storage, read> meshes: Meshes;\n@group(0) @binding(5)\nvar<storage, read> meshlets: Meshlets;\n@group(0) @binding(6)\nvar<storage, read> meshlets_culling: MeshletsCulling;\n@group(0) @binding(7)\nvar<storage, read> culling_result: array<atomic<u32>>;\n\n@group(1) @binding(0)\nvar<storage, read> tlas: BHV;\n@group(1) @binding(1)\nvar<storage, read> bhv: BHV;\n@group(1) @binding(2)\nvar<storage, read> meshes_inverse_matrix: Matrices;\n@group(1) @binding(3)\nvar<storage, read> rays: Rays;\n\nconst HIT_EPSILON: f32 = 0.0001;\nconst INVALID_NODE: i32 = -1;\n\nstruct Result {\n    distance: f32,\n    visibility_id: u32,\n}\n\n\nfn intersect_aabb(ray: ptr<function, Ray>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 \n{     \n    let inverse_dir = 1. / (*ray).direction;\n    let v_min = (aabb_min - (*ray).origin) * inverse_dir;\n    let v_max = (aabb_max - (*ray).origin) * inverse_dir;\n\n    let t_min = min(v_min, v_max);\n    let t_max = max(v_min, v_max);\n\n    let t_near = max(max(t_min.x, t_min.y), max(t_min.x, t_min.z));\n    let t_far = min(min(t_max.x, t_max.y), min(t_max.x, t_max.z));\n\n    if (t_near > t_far || t_far < 0.) { \n        return MAX_FLOAT; \n    }\n    if (t_near > (*ray).t_min) {\n        (*ray).t_max = t_far;\n        return t_near;\n    } else if (t_far < (*ray).t_max) {\n        (*ray).t_max = t_far;\n        return t_far;\n    } \n    return MAX_FLOAT; \n}\n\n\nfn intersect_triangle(ray: ptr<function, Ray>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32\n{\n    let e1 = v1 - v0;\n    let e2 = v2 - v0;\n\n    let p = cross((*ray).direction, e2);\n    let det = dot(e1, p);\n    \n    if (abs(det) < HIT_EPSILON) { return MAX_FLOAT; }\n\n    // Computes Barycentric coordinates.\n    let inv_det = 1. / det;\n    let t1 = (*ray).origin - v0;    \n    let u = dot(t1, p) * inv_det;\n    if (u < 0. || u > 1.) { return MAX_FLOAT; }\n    \n    let q = cross(t1, e1);\n    let v = dot((*ray).direction, q) * inv_det;\n    if (v < 0. || u + v > 1.) { return MAX_FLOAT; }\n\n    let t2 = dot(e2, q) * inv_det;\n    if (t2 < 0.) { return MAX_FLOAT; }\n    return t2;\n}\n\nfn intersect_meshlet_primitive(ray: ptr<function, Ray>, mesh_id: u32, meshlet_id: u32, primitive_id: u32) -> f32 {\n    let mesh = &meshes.data[mesh_id];    \n    let node = &bhv.data[u32((*mesh).bhv_index)];  \n    \n    let meshlet = &meshlets.data[meshlet_id];\n    \n    let index_offset = (*mesh).indices_offset + (*meshlet).indices_offset + (primitive_id * 3u);\n    let i1 = indices.data[index_offset];\n    let i2 = indices.data[index_offset + 1u];\n    let i3 = indices.data[index_offset + 2u];\n\n    let o1 = vertices.data[(*mesh).vertex_offset + i1].position_and_color_offset;\n    let o2 = vertices.data[(*mesh).vertex_offset + i2].position_and_color_offset;\n    let o3 = vertices.data[(*mesh).vertex_offset + i3].position_and_color_offset;\n\n    let v1 = positions.data[o1];\n    let v2 = positions.data[o2];\n    let v3 = positions.data[o3];\n    \n    let oobb_size = (*node).max - (*node).min;\n    \n    let p1 = (*node).min + decode_as_vec3(v1) * oobb_size;\n    let p2 = (*node).min + decode_as_vec3(v2) * oobb_size;\n    let p3 = (*node).min + decode_as_vec3(v3) * oobb_size;\n    \n    return intersect_triangle(ray, p1, p2, p3);\n}\n\nfn is_meshlet_visible(meshlet_id: u32) -> bool\n{\n    let bits = atomicLoad(&culling_result[meshlet_id / 32u]);\n    return (bits & ((1u << meshlet_id) % 32u)) > 0u;\n}\n\nfn intersect_meshlet(ray: ptr<function, Ray>, mesh_id: u32, meshlet_id: u32, far_plane: f32) -> Result {\n    var nearest = far_plane;  \n    var visibility_id = 0u;\n    \n    let mesh = &meshes.data[mesh_id];   \n    let meshlet = &meshlets.data[meshlet_id];\n    let primitive_count = (*meshlet).indices_count / 3u;\n    for(var primitive_id = 0u; primitive_id < primitive_count; primitive_id = primitive_id + 1u)\n    {       \n        let hit = intersect_meshlet_primitive(ray, mesh_id, meshlet_id, primitive_id);\n        visibility_id = select(visibility_id, ((meshlet_id + 1u) << 8u) | primitive_id, hit < nearest);\n        nearest = min(nearest, hit);\n    }\n    return Result(nearest, visibility_id);\n}\n\nfn traverse_bhv_of_meshlets(ray: ptr<function, Ray>, mesh_id: u32, far_plane: f32) -> Result {\n    let mesh = &meshes.data[mesh_id];    \n    var bhv_index = i32((*mesh).bhv_index);    \n    let mesh_bhv_index = bhv_index;\n    var nearest = far_plane;  \n    var visibility_id = 0u;\n\n    while (bhv_index >= 0)\n    { \n        let node = &bhv.data[u32(bhv_index)];   \n        let intersection = intersect_aabb(ray, (*node).min, (*node).max);\n        if (intersection >= nearest) {\n            bhv_index = select((*node).miss, (*node).miss + mesh_bhv_index, (*node).miss >= 0);\n            continue;\n        }\n        if ((*node).reference < 0) {\n            //inner node\n            bhv_index = bhv_index + 1;\n            continue;  \n        }\n        //leaf node\n        let meshlet_id = (*mesh).meshlets_offset + u32((*node).reference);             \n        if (!is_meshlet_visible(meshlet_id)) {\n            bhv_index = select((*node).miss, (*node).miss + mesh_bhv_index, (*node).miss >= 0);\n            continue;\n        }\n        let hit = intersect_meshlet(ray, mesh_id, meshlet_id, nearest);\n        visibility_id = select(visibility_id, hit.visibility_id, hit.distance < nearest);\n        nearest = hit.distance;\n        bhv_index = select((*node).miss, (*node).miss + mesh_bhv_index, (*node).miss >= 0);\n    }\n    return Result(nearest, visibility_id);\n}\n\n\n\n\nfn execute_job(job_index: u32, dimensions: vec2<u32>) -> vec4<f32> {    \n    var ray = rays.data[job_index];\n    var nearest = MAX_FLOAT;  \n    var visibility_id = 0u;\n    \n    var tlas_index = 0;\n    \n    while (tlas_index >= 0)\n    {\n        let node = &tlas.data[u32(tlas_index)];    \n        let intersection = intersect_aabb(&ray, (*node).min, (*node).max);\n        if (intersection >= nearest) {\n            tlas_index = (*node).miss;\n            continue;\n        }\n        if ((*node).reference < 0) {\n            //inner node\n            tlas_index = tlas_index + 1;\n            continue;\n        }\n        //leaf node\n        let mesh_id = u32((*node).reference);\n        let inverse_matrix = &meshes_inverse_matrix.data[mesh_id];    \n        let transformed_origin = (*inverse_matrix) * vec4<f32>(ray.origin, 1.);\n        let transformed_direction = (*inverse_matrix) * vec4<f32>(ray.direction, 0.);\n        var transformed_ray = Ray(transformed_origin.xyz, ray.t_min, transformed_direction.xyz, ray.t_max);\n        let result = traverse_bhv_of_meshlets(&transformed_ray, mesh_id, nearest);\n        visibility_id = select(visibility_id, result.visibility_id, result.distance < nearest);\n        nearest = result.distance;\n        tlas_index = (*node).miss;\n    } \n    return unpack4x8unorm(visibility_id);\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) uv: vec2<f32>,\n};\n\nstruct FragmentOutput {\n    @location(0) color: vec4<f32>,\n};\n\n@vertex\nfn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {\n    //only one triangle, exceeding the viewport size\n    let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));\n    let pos = vec4<f32>(uv * vec2<f32>(2., -2.) + vec2<f32>(-1., 1.), 0., 1.);\n\n    var vertex_out: VertexOutput;\n    vertex_out.clip_position = pos;\n    vertex_out.uv = uv;\n    return vertex_out;\n}\n\n@fragment\nfn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {\n    let pixel = vec2<u32>(u32(v_in.uv.x * f32(data.width)), u32(v_in.uv.y * f32(data.height)));\n\n    let total_job_index = pixel.y * data.width + pixel.x;\n    \n    let texture_color = execute_job(total_job_index, vec2<u32>(data.width, data.height));\n    return texture_color;\n}\n"}