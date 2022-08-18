{"spirv_code":[],"wgsl_code":"fn decode_unorm(i: u32, n: u32) -> f32 {    \n    let scale = f32((1 << n) - 1);\n    if i == 0u {\n        return 0.;\n    } else if i == u32(scale) {\n        return 1.;\n    } else {\n        return (f32(i) - 0.5) / scale;\n    }\n}\n\nfn decode_snorm(i: i32, n: u32) -> f32 {\n    let scale = f32(1 << (n - 1u));\n    return (f32(i) / scale);\n}\n\n\nfn decode_uv(v: u32) -> vec2<f32> {\n    return unpack2x16float(v);\n}\nfn decode_as_vec3(v: u32) -> vec3<f32> {\n    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_unorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn pack_normal(normal: vec3<f32>) -> vec2<f32> {\n    return vec2<f32>(normal.xy * 0.5 + 0.5);\n}\nfn unpack_normal(uv: vec2<f32>) -> vec3<f32> {\n    return vec3<f32>(uv.xy * 2. - 1., sqrt(1.-dot(uv.xy, uv.xy)));\n}\n\nfn quantize_unorm(v: f32, n: u32) -> u32 {\n    let scale = f32((1 << n) - 1);\n    return u32(0.5 + (v * scale));\n}\n\nfn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {\n    let r = quantize_unorm(value.x, 8u) << 24u;\n    let g = quantize_unorm(value.y, 8u) << 16u;\n    let b = quantize_unorm(value.z, 8u) << 8u;\n    let a = quantize_unorm(value.w, 8u);\n    return (r | g | b | a);\n}\nfn unpack_unorm_to_4_f32(color: u32) -> vec4<f32> {\n    return vec4<f32>(\n        f32((color >> 24u) & 255u),\n        f32((color >> 16u) & 255u),\n        f32((color >> 8u) & 255u),\n        f32(color & 255u),\n    );\n}\n\nfn hash(index: u32) -> u32 {\n    var v = index;\n    v = (v + 0x7ed55d16u) + (v << 12u);\n    v = (v ^ 0xc761c23cu) ^ (v >> 19u);\n    v = (v + 0x165667b1u) + (v << 5u);\n    v = (v + 0xd3a2646cu) ^ (v << 9u);\n    v = (v + 0xfd7046c5u) + (v << 3u);\n    v = (v ^ 0xb55a4f09u) ^ (v >> 16u);\n    return v;\n}\n\n// 0-1 from 0-255\nfn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {\n    let cutoff = srgb < vec3<f32>(10.31475);\n    let lower = srgb / vec3<f32>(3294.6);\n    let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));\n    return select(higher, lower, cutoff);\n}\n\n// [u8; 4] SRGB as u32 -> [r, g, b, a]\nfn unpack_color(color: u32) -> vec4<f32> {\n    return vec4<f32>(\n        f32(color & 255u),\n        f32((color >> 8u) & 255u),\n        f32((color >> 16u) & 255u),\n        f32((color >> 24u) & 255u),\n    );\n}\n\nfn extract_scale(m: mat4x4<f32>) -> vec3<f32> {\n    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);\n    let sx = length(s[0]);\n    let sy = length(s[1]);\n    let det = determinant(s);\n    var sz = length(s[2]);\n    if (det < 0.) {\n        sz = -sz;\n    }\n    return vec3<f32>(sx, sy, sz);\n}\n\nfn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> {\n    if (row == 1u) {\n        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);\n    } else if (row == 2u) {\n        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);\n    } else if (row == 3u) {\n        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);\n    } else {        \n        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);\n    }\n}\n\nfn normalize_plane(plane: vec4<f32>) -> vec4<f32> {\n    return (plane / length(plane.xyz));\n}\nlet MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;\nlet MAX_TEXTURE_COORDS_SET: u32 = 4u;\n\nlet TEXTURE_TYPE_BASE_COLOR: u32 = 0u;\nlet TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;\nlet TEXTURE_TYPE_NORMAL: u32 = 2u;\nlet TEXTURE_TYPE_EMISSIVE: u32 = 3u;\nlet TEXTURE_TYPE_OCCLUSION: u32 = 4u;\nlet TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;\nlet TEXTURE_TYPE_DIFFUSE: u32 = 6u;\nlet TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;\nlet TEXTURE_TYPE_COUNT: u32 = 8u;\n\nlet MATERIAL_ALPHA_BLEND_OPAQUE = 0u;\nlet MATERIAL_ALPHA_BLEND_MASK = 1u;\nlet MATERIAL_ALPHA_BLEND_BLEND = 2u;\n\nlet MESH_FLAGS_NONE: u32 = 0u;\nlet MESH_FLAGS_VISIBLE: u32 = 1u;\nlet MESH_FLAGS_OPAQUE: u32 = 2u; // 1 << 1\nlet MESH_FLAGS_TRANSPARENT: u32 = 4u;  // 1 << 2\nlet MESH_FLAGS_WIREFRAME: u32 = 8u; // 1 << 3\nlet MESH_FLAGS_DEBUG: u32 = 16u; // 1 << 4\nlet MESH_FLAGS_UI: u32 = 32u; // 1 << 5\n\nlet CONSTANT_DATA_FLAGS_NONE: u32 = 0u;\nlet CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 2u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE: u32 = 4u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 8u;\n\n\nstruct ConstantData {\n    view: mat4x4<f32>,\n    proj: mat4x4<f32>,\n    inverse_view_proj: mat4x4<f32>,\n    screen_width: f32,\n    screen_height: f32,\n    flags: u32,\n};\n\nstruct DrawVertex {\n    @location(0) position_and_color_offset: u32,\n    @location(1) normal_offset: i32,\n    @location(2) tangent_offset: i32,\n    @location(3) mesh_index: u32,\n    @location(4) uvs_offset: vec4<i32>,\n};\n\nstruct DrawCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_vertex: u32,\n    base_instance: u32,\n};\n\nstruct DrawIndexedCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_index: u32,\n    vertex_offset: i32,\n    base_instance: u32,\n};\n\nstruct DrawMesh {\n    vertex_offset: u32,\n    indices_offset: u32,\n    material_index: i32,\n    mesh_flags: u32,\n    aabb_min: vec3<f32>,\n    meshlet_offset: u32,\n    aabb_max: vec3<f32>,\n    meshlet_count: u32,\n    transform: mat4x4<f32>,\n};\n\nstruct DrawMeshlet {\n    @location(5) mesh_index: u32,\n    @location(6) vertex_offset: u32,\n    @location(7) indices_offset: u32,\n    @location(8) indices_count: u32,\n    @location(9) center_radius: vec4<f32>,\n    @location(10) cone_axis_cutoff: vec4<f32>,\n};\n\n\nstruct LightData {\n    position: vec3<f32>,\n    light_type: u32,\n    color: vec4<f32>,\n    intensity: f32,\n    range: f32,\n    inner_cone_angle: f32,\n    outer_cone_angle: f32,\n};\n\nstruct TextureData {\n    texture_index: u32,\n    layer_index: u32,\n    total_width: f32,\n    total_height: f32,\n    area: vec4<f32>,\n};\n\nstruct DrawMaterial {\n    textures_indices: array<i32, 8>,//TEXTURE_TYPE_COUNT>,\n    textures_coord_set: array<u32, 8>,//TEXTURE_TYPE_COUNT>,\n    roughness_factor: f32,\n    metallic_factor: f32,\n    alpha_cutoff: f32,\n    alpha_mode: u32,\n    base_color: vec4<f32>,\n    emissive_color: vec3<f32>,\n    occlusion_strength: f32,\n    diffuse_color: vec4<f32>,\n    specular_color: vec4<f32>,\n};\n\n\nstruct Lights {\n    data: array<LightData>,\n};\n\nstruct Textures {\n    data: array<TextureData>,\n};\n\nstruct Materials {\n    data: array<DrawMaterial>,\n};\n\nstruct DrawCommands {\n    data: array<DrawCommand>,\n};\n\nstruct RenderCommands {\n    data: array<DrawIndexedCommand>,\n};\n\nstruct Meshes {\n    data: array<DrawMesh>,\n};\n\nstruct Meshlets {\n    data: array<DrawMeshlet>,\n};\n\nstruct Indices {\n    data: array<u32>,\n};\n\nstruct Vertices {\n    data: array<DrawVertex>,\n};\n\nstruct Matrices {\n    data: array<mat4x4<f32>>,\n};\n\nstruct Positions {\n    data: array<u32>,\n};\n\nstruct Colors {\n    data: array<u32>,\n};\n\nstruct Normals {\n    data: array<u32>,\n};\n\nstruct Tangents {\n    data: array<vec4<f32>>,\n};\n\nstruct UVs {\n    data: array<u32>,\n};\n\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) @interpolate(flat) mesh_and_meshlet_ids: vec2<u32>,\n    @location(1) world_pos: vec4<f32>,\n    @location(2) color: vec4<f32>,\n    @location(3) normal: vec3<f32>,\n    @location(4) uv_0: vec2<f32>,\n    @location(5) uv_1: vec2<f32>,\n    @location(6) uv_2: vec2<f32>,\n    @location(7) uv_3: vec2<f32>,\n};\n\nstruct FragmentOutput {\n    @location(0) gbuffer_1: vec4<f32>,  //color        \n    @location(1) gbuffer_2: vec4<f32>,  //normal       \n    @location(2) gbuffer_3: vec4<f32>,  //meshlet_id   \n    @location(3) gbuffer_4: vec4<f32>,  //uv_0         \n    @location(4) gbuffer_5: vec4<f32>,  //uv_1         \n    @location(5) gbuffer_6: vec4<f32>,  //uv_2         \n    @location(6) gbuffer_7: vec4<f32>,  //uv_3         \n};\n\n\n@group(0) @binding(0)\nvar<uniform> constant_data: ConstantData;\n@group(0) @binding(1)\nvar<storage, read> positions: Positions;\n@group(0) @binding(2)\nvar<storage, read> colors: Colors;\n@group(0) @binding(3)\nvar<storage, read> normals: Normals;\n@group(0) @binding(4)\nvar<storage, read> uvs: UVs;\n\n@group(1) @binding(0)\nvar<storage, read> meshes: Meshes;\n@group(1) @binding(1)\nvar<storage, read> materials: Materials;\n@group(1) @binding(2)\nvar<storage, read> textures: Textures;\n@group(1) @binding(3)\nvar<storage, read> meshlets: Meshlets;\n\n@group(2) @binding(0)\nvar default_sampler: sampler;\n@group(2) @binding(1)\nvar unfiltered_sampler: sampler;\n@group(2) @binding(2)\nvar depth_sampler: sampler_comparison;\n\n@group(2) @binding(3)\nvar texture_1: texture_2d_array<f32>;\n@group(2) @binding(4)\nvar texture_2: texture_2d_array<f32>;\n@group(2) @binding(5)\nvar texture_3: texture_2d_array<f32>;\n@group(2) @binding(6)\nvar texture_4: texture_2d_array<f32>;\n@group(2) @binding(7)\nvar texture_5: texture_2d_array<f32>;\n@group(2) @binding(8)\nvar texture_6: texture_2d_array<f32>;\n@group(2) @binding(9)\nvar texture_7: texture_2d_array<f32>;\n@group(2) @binding(10)\nvar texture_8: texture_2d_array<f32>;\n\n\nfn sample_texture(tex_coords_and_texture_index: vec3<f32>) -> vec4<f32> {\n    let texture_data_index = i32(tex_coords_and_texture_index.z);\n    var tex_coords = vec3<f32>(0.0, 0.0, 0.0);\n    if (texture_data_index < 0) {\n        return vec4<f32>(tex_coords, 0.);\n    }\n    let texture = &textures.data[texture_data_index];\n    let atlas_index = (*texture).texture_index;\n    let layer_index = i32((*texture).layer_index);\n\n    tex_coords.x = ((*texture).area.x + tex_coords_and_texture_index.x * (*texture).area.z) / (*texture).total_width;\n    tex_coords.y = ((*texture).area.y + tex_coords_and_texture_index.y * (*texture).area.w) / (*texture).total_height;\n    tex_coords.z = f32(layer_index);\n\n    switch (atlas_index) {\n        default { return textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }\n        case 1u: { return textureSampleLevel(texture_2, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }\n        case 2u: { return textureSampleLevel(texture_3, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }\n        case 3u: { return textureSampleLevel(texture_4, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }\n        case 4u: { return textureSampleLevel(texture_5, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }\n        case 5u: { return textureSampleLevel(texture_6, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }\n        case 6u: { return textureSampleLevel(texture_7, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }\n        case 7u: { return textureSampleLevel(texture_8, default_sampler, tex_coords.xy, layer_index, tex_coords.z); }\n    }\n}\n\n\n\nfn load_texture(tex_coords_and_texture_index: vec3<i32>) -> vec4<f32> {\n    let atlas_index = tex_coords_and_texture_index.z;\n    let layer_index = 0;\n\n    switch (atlas_index) {\n        default { return textureLoad(texture_1, tex_coords_and_texture_index.xy, layer_index, layer_index); }\n        case 1: { return textureLoad(texture_2, tex_coords_and_texture_index.xy, layer_index, layer_index); }\n        case 2: { return textureLoad(texture_3, tex_coords_and_texture_index.xy, layer_index, layer_index); }\n        case 3: { return textureLoad(texture_4, tex_coords_and_texture_index.xy, layer_index, layer_index); }\n        case 4: { return textureLoad(texture_5, tex_coords_and_texture_index.xy, layer_index, layer_index); }\n        case 5: { return textureLoad(texture_6, tex_coords_and_texture_index.xy, layer_index, layer_index); }\n        case 6: { return textureLoad(texture_7, tex_coords_and_texture_index.xy, layer_index, layer_index); }\n        case 7: { return textureLoad(texture_8, tex_coords_and_texture_index.xy, layer_index, layer_index); }\n    }\n}\n\nfn get_uv(uv_set: vec4<u32>, texture_index: u32, coords_set: u32) -> vec3<f32> {\n    var uv = unpack2x16float(uv_set.x);\n    if (coords_set == 1u) {\n        uv = unpack2x16float(uv_set.y);\n    } else if (coords_set == 2u) {\n        uv = unpack2x16float(uv_set.z);\n    } else if (coords_set == 3u) {\n        uv = unpack2x16float(uv_set.w);\n    }\n    return vec3<f32>(uv, f32(texture_index));\n}\n\n\nfn compute_world_position_from_depth(uv: vec2<f32>, depth_texture_index: u32) -> vec3<f32> {\n  let clip_position = vec4<f32>(uv * 2. - 1., sample_texture(vec3<f32>(uv, f32(depth_texture_index))).r * 2. - 1., 1.);\n  let homogeneous = constant_data.inverse_view_proj * clip_position;\n  return homogeneous.xyz / homogeneous.w;\n}\nfn has_texture(material_index: u32, texture_type: u32) -> bool {\n    if (materials.data[material_index].textures_indices[texture_type] >= 0) {\n        return true;\n    }\n    return false;\n}\n\nfn material_texture_index(material_index: u32, texture_type: u32) -> i32 {\n    let material = &materials.data[material_index];\n    return (*material).textures_indices[texture_type];\n}\n\nfn material_texture_coord_set(material_index: u32, texture_type: u32) -> u32 {\n    let material = &materials.data[material_index];\n    return (*material).textures_coord_set[texture_type];\n}\n\nfn compute_uvs(material_index: u32, texture_type: u32, uv_set: vec4<u32>) -> vec3<f32> {\n    let texture_id = material_texture_index(material_index, texture_type);\n    let coords_set = material_texture_coord_set(material_index, texture_type);  \n    let uv = get_uv(uv_set, u32(texture_id), coords_set);\n    return uv;\n}\n\nfn sample_material_texture(material_index: u32, texture_type: u32, uv_set: vec4<u32>) -> vec4<f32> {\n    let uv = compute_uvs(material_index, texture_type, uv_set);\n    return sample_texture(uv);\n}\n\n\n@vertex\nfn vs_main(\n    @builtin(vertex_index) vertex_index: u32,\n    @builtin(instance_index) meshlet_id: u32,\n    v_in: DrawVertex,\n) -> VertexOutput {\n    let mvp = constant_data.proj * constant_data.view;\n\n    let mesh_id = u32(meshlets.data[meshlet_id].mesh_index);\n    let mesh = &meshes.data[mesh_id];\n\n    let aabb_size = abs((*mesh).aabb_max - (*mesh).aabb_min);\n    \n    let p = (*mesh).aabb_min + decode_as_vec3(positions.data[v_in.position_and_color_offset]) * aabb_size;\n    let world_position = (*mesh).transform * vec4<f32>(p, 1.0);\n    let color = unpack_unorm_to_4_f32(colors.data[v_in.position_and_color_offset]);\n    \n    var vertex_out: VertexOutput;\n    vertex_out.clip_position = mvp * world_position;\n    vertex_out.mesh_and_meshlet_ids = vec2<u32>(mesh_id, meshlet_id);\n    vertex_out.world_pos = world_position;\n    vertex_out.color = color;\n    vertex_out.normal = decode_as_vec3(normals.data[v_in.normal_offset]); \n    vertex_out.uv_0 = unpack2x16float(uvs.data[v_in.uvs_offset.x]);\n    vertex_out.uv_1 = unpack2x16float(uvs.data[v_in.uvs_offset.y]);\n    vertex_out.uv_2 = unpack2x16float(uvs.data[v_in.uvs_offset.z]);\n    vertex_out.uv_3 = unpack2x16float(uvs.data[v_in.uvs_offset.w]);\n\n    return vertex_out;\n}\n\n@fragment\nfn fs_main(\n    v_in: VertexOutput,\n) -> FragmentOutput {    \n    var fragment_out: FragmentOutput;\n\n    let mesh_id = u32(v_in.mesh_and_meshlet_ids.x);\n    let mesh = &meshes.data[mesh_id];\n    let material_id = u32((*mesh).material_index);\n    let uv_set = vec4<u32>(\n        pack2x16float(v_in.uv_0),\n        pack2x16float(v_in.uv_1),\n        pack2x16float(v_in.uv_2),\n        pack2x16float(v_in.uv_3)\n    );\n    // Retrieve the tangent space transform\n    var n = normalize(v_in.normal.xyz); \n    if (has_texture(material_id, TEXTURE_TYPE_NORMAL)) {    \n        let uv = compute_uvs(material_id, TEXTURE_TYPE_NORMAL, uv_set);    \n        // get edge vectors of the pixel triangle \n        let dp1 = dpdx( v_in.world_pos.xyz ); \n        let dp2 = dpdy( v_in.world_pos.xyz ); \n        let duv1 = dpdx( uv.xy ); \n        let duv2 = dpdy( uv.xy );   \n        // solve the linear system \n        let dp2perp = cross( dp2, n ); \n        let dp1perp = cross( n, dp1 ); \n        let tangent = dp2perp * duv1.x + dp1perp * duv2.x; \n        let bitangent = dp2perp * duv1.y + dp1perp * duv2.y;\n        let t = normalize(tangent);\n        let b = normalize(bitangent); \n        let tbn = mat3x3<f32>(t, b, n);\n        let normal = sample_texture(uv);\n        n = tbn * (2.0 * normal.rgb - vec3<f32>(1.0));\n        n = normalize(n);\n    }\n\n    fragment_out.gbuffer_1 = v_in.color;\n    fragment_out.gbuffer_2 = unpack4x8unorm(pack2x16float(pack_normal(n)));\n    fragment_out.gbuffer_3 = unpack4x8unorm(v_in.mesh_and_meshlet_ids.y + 1u);\n    fragment_out.gbuffer_4 = unpack4x8unorm(pack2x16float(v_in.uv_0));\n    fragment_out.gbuffer_5 = unpack4x8unorm(pack2x16float(v_in.uv_1));\n    fragment_out.gbuffer_6 = unpack4x8unorm(pack2x16float(v_in.uv_2));\n    fragment_out.gbuffer_7 = unpack4x8unorm(pack2x16float(v_in.uv_3));\n    \n    return fragment_out;\n}\n"}