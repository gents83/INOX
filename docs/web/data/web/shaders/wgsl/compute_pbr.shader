{"spirv_code":[],"wgsl_code":"fn decode_unorm(i: u32, n: u32) -> f32 {    \n    let scale = f32((1 << n) - 1);\n    if (i == 0u) {\n        return 0.;\n    } else if (i == u32(scale)) {\n        return 1.;\n    } else {\n        return (f32(i) - 0.5) / scale;\n    }\n}\n\nfn decode_snorm(i: i32, n: u32) -> f32 {\n    let scale = f32(1 << (n - 1u));\n    return (f32(i) / scale);\n}\n\n\nfn decode_uv(v: u32) -> vec2<f32> {\n    return unpack2x16float(v);\n}\nfn decode_as_vec3(v: u32) -> vec3<f32> {\n    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_unorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn pack_normal(normal: vec3<f32>) -> vec2<f32> {\n    return vec2<f32>(normal.xy * 0.5 + 0.5);\n}\nfn unpack_normal(uv: vec2<f32>) -> vec3<f32> {\n    return vec3<f32>(uv.xy * 2. - 1., sqrt(1.-dot(uv.xy, uv.xy)));\n}\n\nfn quantize_unorm(v: f32, n: u32) -> u32 {\n    let scale = f32((1 << n) - 1);\n    return u32(0.5 + (v * scale));\n}\n\nfn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {\n    let r = quantize_unorm(value.x, 8u) << 24u;\n    let g = quantize_unorm(value.y, 8u) << 16u;\n    let b = quantize_unorm(value.z, 8u) << 8u;\n    let a = quantize_unorm(value.w, 8u);\n    return (r | g | b | a);\n}\nfn unpack_unorm_to_4_f32(color: u32) -> vec4<f32> {\n    return vec4<f32>(\n        f32((color >> 24u) & 255u),\n        f32((color >> 16u) & 255u),\n        f32((color >> 8u) & 255u),\n        f32(color & 255u),\n    );\n}\n\nfn hash(index: u32) -> u32 {\n    var v = index;\n    v = (v + 0x7ed55d16u) + (v << 12u);\n    v = (v ^ 0xc761c23cu) ^ (v >> 19u);\n    v = (v + 0x165667b1u) + (v << 5u);\n    v = (v + 0xd3a2646cu) ^ (v << 9u);\n    v = (v + 0xfd7046c5u) + (v << 3u);\n    v = (v ^ 0xb55a4f09u) ^ (v >> 16u);\n    return v;\n}\n\n// 0-1 from 0-255\nfn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {\n    let cutoff = srgb < vec3<f32>(10.31475);\n    let lower = srgb / vec3<f32>(3294.6);\n    let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));\n    return select(higher, lower, cutoff);\n}\n\n// [u8; 4] SRGB as u32 -> [r, g, b, a]\nfn unpack_color(color: u32) -> vec4<f32> {\n    return vec4<f32>(\n        f32(color & 255u),\n        f32((color >> 8u) & 255u),\n        f32((color >> 16u) & 255u),\n        f32((color >> 24u) & 255u),\n    );\n}\n\nfn extract_scale(m: mat4x4<f32>) -> vec3<f32> {\n    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);\n    let sx = length(s[0]);\n    let sy = length(s[1]);\n    let det = determinant(s);\n    var sz = length(s[2]);\n    if (det < 0.) {\n        sz = -sz;\n    }\n    return vec3<f32>(sx, sy, sz);\n}\n\nfn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> {\n    if (row == 1u) {\n        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);\n    } else if (row == 2u) {\n        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);\n    } else if (row == 3u) {\n        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);\n    } else {        \n        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);\n    }\n}\n\nfn normalize_plane(plane: vec4<f32>) -> vec4<f32> {\n    return (plane / length(plane.xyz));\n}\n\nfn rotate_vector(v: vec3<f32>, orientation: vec4<f32>) -> vec3<f32> {\n    return v + 2. * cross(orientation.xyz, cross(orientation.xyz, v) + orientation.w * v);\n}\nfn transform_vector(v: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> vec3<f32> {\n    return rotate_vector(v, orientation) * scale + position;\n}\nlet MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;\nlet MAX_TEXTURE_COORDS_SET: u32 = 4u;\n\nlet TEXTURE_TYPE_BASE_COLOR: u32 = 0u;\nlet TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;\nlet TEXTURE_TYPE_NORMAL: u32 = 2u;\nlet TEXTURE_TYPE_EMISSIVE: u32 = 3u;\nlet TEXTURE_TYPE_OCCLUSION: u32 = 4u;\nlet TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;\nlet TEXTURE_TYPE_DIFFUSE: u32 = 6u;\nlet TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;\nlet TEXTURE_TYPE_COUNT: u32 = 8u;\n\nlet MATERIAL_ALPHA_BLEND_OPAQUE = 0u;\nlet MATERIAL_ALPHA_BLEND_MASK = 1u;\nlet MATERIAL_ALPHA_BLEND_BLEND = 2u;\n\nlet MESH_FLAGS_NONE: u32 = 0u;\nlet MESH_FLAGS_VISIBLE: u32 = 1u;\nlet MESH_FLAGS_OPAQUE: u32 = 2u; // 1 << 1\nlet MESH_FLAGS_TRANSPARENT: u32 = 4u;  // 1 << 2\nlet MESH_FLAGS_WIREFRAME: u32 = 8u; // 1 << 3\nlet MESH_FLAGS_DEBUG: u32 = 16u; // 1 << 4\nlet MESH_FLAGS_UI: u32 = 32u; // 1 << 5\n\nlet CONSTANT_DATA_FLAGS_NONE: u32 = 0u;\nlet CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 2u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE: u32 = 4u;\nlet CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 8u;\n\n\nstruct ConstantData {\n    view: mat4x4<f32>,\n    proj: mat4x4<f32>,\n    inverse_view_proj: mat4x4<f32>,\n    screen_width: f32,\n    screen_height: f32,\n    flags: u32,\n};\n\nstruct Vertex {\n    @location(0) position_and_color_offset: u32,\n    @location(1) normal_offset: i32,\n    @location(2) tangent_offset: i32,\n    @location(3) mesh_index: u32,\n    @location(4) uvs_offset: vec4<i32>,\n};\n\nstruct DrawCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_vertex: u32,\n    base_instance: u32,\n};\n\nstruct DrawIndexedCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_index: u32,\n    vertex_offset: i32,\n    base_instance: u32,\n};\n\nstruct Mesh {\n    vertex_offset: u32,\n    indices_offset: u32,\n    material_index: i32,\n    mesh_flags: u32,\n    position: vec3<f32>,\n    meshlets_offset: u32,\n    scale: vec3<f32>,\n    meshlets_count: u32,\n    orientation: vec4<f32>,\n};\n\nstruct Meshlet {\n    @location(5) mesh_index: u32,\n    @location(6) aabb_index: u32,\n    @location(7) indices_offset: u32,\n    @location(8) indices_count: u32,\n    @location(9) cone_axis_cutoff: vec4<f32>,\n};\n\nstruct AABB {\n    min: vec3<f32>,\n    child_start: i32,\n    max: vec3<f32>,\n    parent_or_count: i32,\n};\n\n\nstruct LightData {\n    position: vec3<f32>,\n    light_type: u32,\n    color: vec4<f32>,\n    intensity: f32,\n    range: f32,\n    inner_cone_angle: f32,\n    outer_cone_angle: f32,\n};\n\nstruct TextureData {\n    texture_index: u32,\n    layer_index: u32,\n    total_width: f32,\n    total_height: f32,\n    area: vec4<f32>,\n};\n\nstruct Material {\n    textures_indices: array<i32, 8>,//TEXTURE_TYPE_COUNT>,\n    textures_coord_set: array<u32, 8>,//TEXTURE_TYPE_COUNT>,\n    roughness_factor: f32,\n    metallic_factor: f32,\n    alpha_cutoff: f32,\n    alpha_mode: u32,\n    base_color: vec4<f32>,\n    emissive_color: vec3<f32>,\n    occlusion_strength: f32,\n    diffuse_color: vec4<f32>,\n    specular_color: vec4<f32>,\n};\n\n\nstruct Lights {\n    data: array<LightData>,\n};\n\nstruct Textures {\n    data: array<TextureData>,\n};\n\nstruct Materials {\n    data: array<Material>,\n};\n\nstruct DrawCommands {\n    data: array<DrawCommand>,\n};\n\nstruct DrawIndexedCommands {\n    data: array<DrawIndexedCommand>,\n};\n\nstruct Meshes {\n    data: array<Mesh>,\n};\n\nstruct Meshlets {\n    data: array<Meshlet>,\n};\n\nstruct Indices {\n    data: array<u32>,\n};\n\nstruct Vertices {\n    data: array<Vertex>,\n};\n\nstruct Matrices {\n    data: array<mat4x4<f32>>,\n};\n\nstruct Positions {\n    data: array<u32>,\n};\n\nstruct Colors {\n    data: array<u32>,\n};\n\nstruct Normals {\n    data: array<u32>,\n};\n\nstruct Tangents {\n    data: array<vec4<f32>>,\n};\n\nstruct UVs {\n    data: array<u32>,\n};\n\nstruct AABBs {\n    data: array<AABB>,\n};\n\n\n\nstruct PbrData {\n    width: u32,\n    height: u32,\n    visibility_buffer_index: u32,\n    _padding: u32,\n};\n\n\n@group(0) @binding(0)\nvar<uniform> constant_data: ConstantData;\n@group(0) @binding(1)\nvar<uniform> pbr_data: PbrData;\n@group(0) @binding(2)\nvar<storage, read> indices: Indices;\n@group(0) @binding(3)\nvar<storage, read> vertices: Vertices;\n@group(0) @binding(4)\nvar<storage, read> positions: Positions;\n@group(0) @binding(5)\nvar<storage, read> colors: Colors;\n@group(0) @binding(6)\nvar<storage, read> normals: Normals;\n@group(0) @binding(7)\nvar<storage, read> uvs: UVs;\n\n@group(1) @binding(0)\nvar<storage, read> meshes: Meshes;\n@group(1) @binding(1)\nvar<storage, read> meshlets: Meshlets;\n@group(1) @binding(2)\nvar<storage, read> materials: Materials;\n@group(1) @binding(3)\nvar<storage, read> textures: Textures;\n@group(1) @binding(4)\nvar<storage, read> lights: Lights;\n@group(1) @binding(5)\nvar<storage, read> meshes_aabb: AABBs;\n\n@group(3) @binding(0)\nvar visibility_buffer_texture: texture_2d<f32>;\n@group(3) @binding(1)\nvar render_target: texture_storage_2d<rgba8unorm, read_write>;\n\n\n\n@group(2) @binding(0)\nvar default_sampler: sampler;\n\n@group(2) @binding(1)\nvar texture_1: texture_2d_array<f32>;\n@group(2) @binding(2)\nvar texture_2: texture_2d_array<f32>;\n@group(2) @binding(3)\nvar texture_3: texture_2d_array<f32>;\n@group(2) @binding(4)\nvar texture_4: texture_2d_array<f32>;\n@group(2) @binding(5)\nvar texture_5: texture_2d_array<f32>;\n@group(2) @binding(6)\nvar texture_6: texture_2d_array<f32>;\n@group(2) @binding(7)\nvar texture_7: texture_2d_array<f32>;\n\n\nfn sample_texture(tex_coords_and_texture_index: vec3<f32>) -> vec4<f32> {\n    let texture_data_index = i32(tex_coords_and_texture_index.z);\n    var v = vec4<f32>(0.);\n    var tex_coords = vec3<f32>(0.0, 0.0, 0.0);\n    if (texture_data_index < 0) {\n        return v;\n    }\n    let texture = &textures.data[texture_data_index];\n    let atlas_index = (*texture).texture_index;\n    let layer_index = i32((*texture).layer_index);\n\n    tex_coords.x = ((*texture).area.x + tex_coords_and_texture_index.x * (*texture).area.z) / (*texture).total_width;\n    tex_coords.y = ((*texture).area.y + tex_coords_and_texture_index.y * (*texture).area.w) / (*texture).total_height;\n    tex_coords.z = f32(layer_index);\n\n    switch (atlas_index) {\n        case 0u: { v = textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, 0.); }\n        case 1u: { v = textureSampleLevel(texture_2, default_sampler, tex_coords.xy, layer_index, 0.); }\n        case 2u: { v = textureSampleLevel(texture_3, default_sampler, tex_coords.xy, layer_index, 0.); }\n        case 3u: { v = textureSampleLevel(texture_4, default_sampler, tex_coords.xy, layer_index, 0.); }\n        case 4u: { v = textureSampleLevel(texture_5, default_sampler, tex_coords.xy, layer_index, 0.); }\n        case 5u: { v = textureSampleLevel(texture_6, default_sampler, tex_coords.xy, layer_index, 0.); }\n        case 6u: { v = textureSampleLevel(texture_7, default_sampler, tex_coords.xy, layer_index, 0.); }\n        default { v = textureSampleLevel(texture_1, default_sampler, tex_coords.xy, layer_index, 0.); }\n    };\n    return v;\n}\n\n\nfn get_uv(uv_set: vec4<u32>, texture_index: u32, coords_set: u32) -> vec3<f32> {\n    var uv = vec2<f32>(0., 0.);\n    switch (coords_set) {\n        case 1u: { uv = unpack2x16float(uv_set.y); }\n        case 2u: { uv = unpack2x16float(uv_set.z); }\n        case 3u: { uv = unpack2x16float(uv_set.w); }\n        default { uv = unpack2x16float(uv_set.x); }\n    }\n    return vec3<f32>(uv, f32(texture_index));\n}\n\nfn has_texture(material_index: u32, texture_type: u32) -> bool {\n    if (materials.data[material_index].textures_indices[texture_type] >= 0) {\n        return true;\n    }\n    return false;\n}\n\nfn material_texture_index(material_index: u32, texture_type: u32) -> i32 {\n    let material = &materials.data[material_index];\n    let texture_index = (*material).textures_indices[texture_type];\n    if (texture_index < 0) {\n        return 0;\n    }\n    return texture_index;\n}\n\nfn material_texture_coord_set(material_index: u32, texture_type: u32) -> u32 {\n    let material = &materials.data[material_index];\n    return (*material).textures_coord_set[texture_type];\n}\n\nfn compute_uvs(material_index: u32, texture_type: u32, uv_set: vec4<u32>) -> vec3<f32> {\n    let texture_id = material_texture_index(material_index, texture_type);\n    let coords_set = material_texture_coord_set(material_index, texture_type);  \n    let uv = get_uv(uv_set, u32(texture_id), coords_set);\n    return uv;\n}\n\nfn sample_material_texture(material_index: u32, texture_type: u32, uv_set: vec4<u32>) -> vec4<f32> {\n    let uv = compute_uvs(material_index, texture_type, uv_set);\n    return sample_texture(uv);\n}\n\nstruct Derivatives {\n    dx: vec3<f32>,\n    dy: vec3<f32>,\n}\n\nfn compute_barycentrics(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, p: vec2<f32>) -> vec3<f32> {\n    let v0 = b - a;\n    let v1 = c - a;\n    let v2 = p - a;\n    \n    let d00 = dot(v0, v0);    \n    let d01 = dot(v0, v1);    \n    let d11 = dot(v1, v1);\n    let d20 = dot(v2, v0);\n    let d21 = dot(v2, v1);\n    \n    let inv_denom = 1. / (d00 * d11 - d01 * d01);    \n    let v = (d11 * d20 - d01 * d21) * inv_denom;    \n    let w = (d00 * d21 - d01 * d20) * inv_denom;    \n    let u = 1. - v - w;\n\n    return vec3 (u,v,w);\n}\n// Engel's barycentric coord partial derivs function. Follows equation from [Schied][Dachsbacher]\n// Computes the partial derivatives of point's barycentric coordinates from the projected screen space vertices\nfn compute_partial_derivatives(v0: vec2<f32>, v1: vec2<f32>, v2: vec2<f32>) -> Derivatives\n{\n    let d = 1. / determinant(mat2x2<f32>(v2-v1, v0-v1));\n    \n    var deriv: Derivatives;\n    deriv.dx = vec3<f32>(v1.y - v2.y, v2.y - v0.y, v0.y - v1.y) * d;\n    deriv.dy = vec3<f32>(v2.x - v1.x, v0.x - v2.x, v1.x - v0.x) * d;\n    return deriv;\n}\n\n// Interpolate 2D attributes using the partial derivatives and generates dx and dy for texture sampling.\nfn interpolate_2d_attribute(a0: vec2<f32>, a1: vec2<f32>, a2: vec2<f32>, deriv: Derivatives, delta: vec2<f32>) -> vec2<f32>\n{\n\tlet attr0 = vec3<f32>(a0.x, a1.x, a2.x);\n\tlet attr1 = vec3<f32>(a0.y, a1.y, a2.y);\n\tlet attribute_x = vec2<f32>(dot(deriv.dx, attr0), dot(deriv.dx, attr1));\n\tlet attribute_y = vec2<f32>(dot(deriv.dy, attr0), dot(deriv.dy, attr1));\n\tlet attribute_s = a0;\n\t\n\treturn (attribute_s + delta.x * attribute_x + delta.y * attribute_y);\n}\n\n// Interpolate vertex attributes at point 'd' using the partial derivatives\nfn interpolate_3d_attribute(a0: vec3<f32>, a1: vec3<f32>, a2: vec3<f32>, deriv: Derivatives, delta: vec2<f32>) -> vec3<f32>\n{\n\tlet attr0 = vec3<f32>(a0.x, a1.x, a2.x);\n\tlet attr1 = vec3<f32>(a0.y, a1.y, a2.y);\n\tlet attr2 = vec3<f32>(a0.z, a1.z, a2.z);\n    let attributes = mat3x3<f32>(a0, a1, a2);\n\tlet attribute_x = attributes * deriv.dx;\n\tlet attribute_y = attributes * deriv.dy;\n\tlet attribute_s = a0;\n\t\n\treturn (attribute_s + delta.x * attribute_x + delta.y * attribute_y);\n}\n\nlet PI: f32 = 3.141592653589793;\nlet AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.75, 0.75, 0.75);\nlet AMBIENT_INTENSITY = 0.45;\nlet NULL_VEC4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);\nlet MIN_ROUGHNESS = 0.04;\n\nfn compute_alpha(material_index: u32, vertex_color_alpha: f32) -> f32 {\n    let material = &materials.data[material_index];\n    // NOTE: the spec mandates to ignore any alpha value in 'OPAQUE' mode\n    var alpha = 1.;\n    if ((*material).alpha_mode == MATERIAL_ALPHA_BLEND_OPAQUE) {\n        alpha = 1.;\n    } else if ((*material).alpha_mode == MATERIAL_ALPHA_BLEND_MASK) {\n        if (alpha >= (*material).alpha_cutoff) {\n            // NOTE: If rendering as masked alpha and >= the cutoff, render as fully opaque\n            alpha = 1.;\n        } else {\n            // NOTE: output_color.a < material.alpha_cutoff should not is not rendered\n            // NOTE: This and any other discards mean that early-z testing cannot be done!\n            alpha = -1.;\n        }\n    } else if ((*material).alpha_mode == MATERIAL_ALPHA_BLEND_BLEND) {\n        alpha = min((*material).base_color.a, vertex_color_alpha);\n    }\n    return alpha;\n}\n\n\n// The following equation models the Fresnel reflectance term of the spec equation (aka F())\n// Implementation of fresnel from [4], Equation 15\nfn specular_reflection(reflectance0: vec3<f32>, reflectance90: vec3<f32>, VdotH: f32) -> vec3<f32> {\n    return reflectance0 + (reflectance90 - reflectance0) * pow(clamp(1.0 - VdotH, 0.0, 1.0), 5.0);\n}\n// This calculates the specular geometric attenuation (aka G()),\n// where rougher material will reflect less light back to the viewer.\n// This implementation is based on [1] Equation 4, and we adopt their modifications to\n// alphaRoughness as input as originally proposed in [2].\nfn geometric_occlusion(alpha_roughness: f32, NdotL: f32, NdotV: f32) -> f32 {\n    let r = alpha_roughness;\n\n    let attenuationL = 2.0 * NdotL / (NdotL + sqrt(r * r + (1.0 - r * r) * (NdotL * NdotL)));\n    let attenuationV = 2.0 * NdotV / (NdotV + sqrt(r * r + (1.0 - r * r) * (NdotV * NdotV)));\n    return attenuationL * attenuationV;\n}\n\n// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())\n// Implementation from \"Average Irregularity Representation of a Roughened Surface for Ray Reflection\" by T. S. Trowbridge, and K. P. Reitz\n// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.\nfn microfacet_distribution(alpha_roughness: f32, NdotH: f32) -> f32 {\n    let roughnessSq = alpha_roughness * alpha_roughness;\n    let f = (NdotH * roughnessSq - NdotH) * NdotH + 1.0;\n    return roughnessSq / (PI * f * f);\n}\n\nfn compute_brdf(world_pos: vec3<f32>, n: vec3<f32>, material_id: u32, color: vec4<f32>, uv_set: vec4<u32>,) -> vec4<f32> {\n    let material = &materials.data[material_id];\n    var perceptual_roughness = (*material).roughness_factor;\n    var metallic = (*material).metallic_factor;\n    if (has_texture(material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {\n        let t = sample_material_texture(material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS, uv_set);\n        metallic = metallic * t.b;\n        perceptual_roughness = perceptual_roughness * t.g;\n    }\n    perceptual_roughness = clamp(perceptual_roughness, MIN_ROUGHNESS, 1.0);\n    metallic = clamp(metallic, 0.0, 1.0);\n    // Roughness is authored as perceptual roughness; as is convention,\n    // convert to material roughness by squaring the perceptual roughness [2].\n    let alpha_roughness = perceptual_roughness * perceptual_roughness;\n\n    var ao = 1.0;\n    var occlusion_strength = 1.;\n    if (has_texture(material_id, TEXTURE_TYPE_OCCLUSION)) {\n        let t = sample_material_texture(material_id, TEXTURE_TYPE_OCCLUSION, uv_set);\n        ao = ao * t.r;\n        occlusion_strength = (*material).occlusion_strength;\n    }\n    var emissive_color = vec3<f32>(0., 0., 0.);\n    if (has_texture(material_id, TEXTURE_TYPE_EMISSIVE)) {\n        let t = sample_material_texture(material_id, TEXTURE_TYPE_EMISSIVE, uv_set);\n        emissive_color = t.rgb * (*material).emissive_color;\n    }\n\n    let f0 = vec3<f32>(0.04, 0.04, 0.04);\n    var diffuse_color = color.rgb * (vec3<f32>(1., 1., 1.) - f0);\n    diffuse_color = diffuse_color * (1.0 - metallic);\n    let specular_color = mix(f0, color.rgb, metallic);        \n\n    // Compute reflectance.\n    let reflectance = max(max(specular_color.r, specular_color.g), specular_color.b);\n\n    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.\n    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.\n    let reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);\n    let specular_environmentR0 = specular_color.rgb;\n    let specular_environmentR90 = vec3<f32>(1., 1., 1.) * reflectance90;\n\n    let view_pos = constant_data.view[3].xyz;\n    let v = normalize(view_pos-world_pos);\n\n    let NdotV = clamp(abs(dot(n, v)), 0.0001, 1.0);\n    let reflection = reflect(-v, n);\n    \n    var final_color = color.rgb * AMBIENT_COLOR * AMBIENT_INTENSITY;\n    final_color = mix(final_color, final_color * ao, occlusion_strength);\n    final_color = final_color + emissive_color;\n\n    let num_lights = arrayLength(&lights.data);\n    for (var i = 0u; i < num_lights; i++ ) {\n        let light = &lights.data[i];\n        if ((*light).light_type == 0u) {\n            break;\n        }\n        let dir = (*light).position - world_pos;\n        let l = normalize(dir);                 // Vector from surface point to light\n        let h = normalize(l + v);               // Half vector between both l and v\n        let dist = length(dir);                 // Distance from surface point to light\n\n        let NdotL = clamp(dot(n, l), 0.0001, 1.0);\n        let NdotH = clamp(dot(n, h), 0.0, 1.0);\n        let LdotH = clamp(dot(l, h), 0.0, 1.0);\n        let VdotH = clamp(dot(v, h), 0.0, 1.0);\n        \n        // Calculate the shading terms for the microfacet specular shading model\n        let F = specular_reflection(specular_environmentR0, specular_environmentR90, VdotH);\n        let G = geometric_occlusion(alpha_roughness, NdotL, NdotV);\n        let D = microfacet_distribution(alpha_roughness, NdotH);\n\n        // Calculation of analytical lighting contribution\n        var intensity = max(200., (*light).intensity);\n        intensity = intensity / (4. * PI);\n        let range = max(8., (*light).range);\n        let light_contrib = (1. - min(dist / range, 1.)) * intensity;\n        let diffuse_contrib = (1. - F) * diffuse_color / PI;\n        let spec_contrib = F * G * D / (4.0 * NdotL * NdotV);\n        var light_color = NdotL * (*light).color.rgb * (diffuse_contrib + spec_contrib);\n        light_color = light_color * light_contrib;\n\n        final_color = final_color + light_color;\n    }\n    \n    return vec4<f32>(final_color, color.a);\n}\n\n\n@compute\n@workgroup_size(8, 4, 1)\nfn main(\n    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, \n    @builtin(local_invocation_index) local_invocation_index: u32, \n    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, \n    @builtin(workgroup_id) workgroup_id: vec3<u32>\n) {\n    for (var i = 0u; i < 8u; i++) {     \n        for (var j = 0u; j < 8u; j++) {            \n            let pixel = vec3<i32>(i32(global_invocation_id.x * 8u + i), i32(global_invocation_id.y * 8u + j), i32(pbr_data.visibility_buffer_index));\n            if (pixel.x >= i32(pbr_data.width) || pixel.y >= i32(pbr_data.height))\n            {\n                continue;\n            }\n            \n            var color = vec4<f32>(0., 0., 0., 0.);\n            let visibility_output = textureLoad(visibility_buffer_texture, pixel.xy, 0);\n            let visibility_id = pack4x8unorm(visibility_output);\n            if ((visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {\n                textureStore(render_target, pixel.xy, color);\n                continue;\n            }\n\n            let meshlet_id = (visibility_id >> 8u) - 1u; \n            let primitive_id = visibility_id & 255u;\n\n            let meshlet = &meshlets.data[meshlet_id];\n            let mesh_id = (*meshlet).mesh_index;\n\n            if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS) != 0u) {\n                let c = hash(meshlet_id);\n                color = vec4<f32>(vec3<f32>(\n                    f32(c & 255u), \n                    f32((c >> 8u) & 255u), \n                    f32((c >> 16u) & 255u)) / 255., \n                    1.\n                );\n            } else {\n                let mesh = &meshes.data[mesh_id];\n                let material_id = u32((*mesh).material_index);\n\n                let mvp = constant_data.proj * constant_data.view;\n\n                var screen_pixel = vec2<f32>(f32(pixel.x), f32(pixel.y));\n                screen_pixel = screen_pixel / vec2<f32>(f32(pbr_data.width), f32(pbr_data.height));\n                screen_pixel.y = 1. - screen_pixel.y;\n                \n                let index_offset = (*mesh).indices_offset + (*meshlet).indices_offset + 3u * primitive_id;\n                let i1 = indices.data[index_offset];\n                let i2 = indices.data[index_offset + 1u];\n                let i3 = indices.data[index_offset + 2u];\n\n                let vertex_offset = (*mesh).vertex_offset;\n                let v1 = &vertices.data[vertex_offset + i1];\n                let v2 = &vertices.data[vertex_offset + i2];\n                let v3 = &vertices.data[vertex_offset + i3];\n\n                let aabb = &meshes_aabb.data[mesh_id];\n                let aabb_size = abs((*aabb).max - (*aabb).min);\n\n                let vp1 = (*aabb).min + decode_as_vec3(positions.data[(*v1).position_and_color_offset]) * aabb_size;\n                let vp2 = (*aabb).min + decode_as_vec3(positions.data[(*v2).position_and_color_offset]) * aabb_size;\n                let vp3 = (*aabb).min + decode_as_vec3(positions.data[(*v3).position_and_color_offset]) * aabb_size;\n\n                var p1 = mvp * vec4<f32>(transform_vector(vp1, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);\n                var p2 = mvp * vec4<f32>(transform_vector(vp2, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);\n                var p3 = mvp * vec4<f32>(transform_vector(vp3, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);\n\n                // Calculate the inverse of w, since it's going to be used several times\n                let one_over_w = 1. / vec3<f32>(p1.w, p2.w, p3.w);\n                p1 = (p1 * one_over_w.x + 1.) * 0.5;\n                p2 = (p2 * one_over_w.y + 1.) * 0.5;\n                p3 = (p3 * one_over_w.z + 1.) * 0.5;\n\n                // Get delta vector that describes current screen point relative to vertex 0\n                let delta = screen_pixel + -p1.xy;\n                let barycentrics = compute_barycentrics(p1.xy, p2.xy, p3.xy, screen_pixel.xy);\n                let deriv = compute_partial_derivatives(p1.xy, p2.xy, p3.xy);\n\n                let c1 = unpack_unorm_to_4_f32(u32(colors.data[(*v1).position_and_color_offset])) / 255.;\n                let c2 = unpack_unorm_to_4_f32(u32(colors.data[(*v2).position_and_color_offset])) / 255.;\n                let c3 = unpack_unorm_to_4_f32(u32(colors.data[(*v3).position_and_color_offset])) / 255.;\n\n                let vertex_color = barycentrics.x * c1 + barycentrics.y * c2 + barycentrics.z * c3;        \n                let alpha = compute_alpha(material_id, vertex_color.a);\n                if alpha < 0. {\n                    textureStore(render_target, pixel.xy, color);\n                    continue;\n                }        \n\n                let uv0_1 = unpack2x16float(uvs.data[(*v1).uvs_offset[0]]);\n                let uv0_2 = unpack2x16float(uvs.data[(*v2).uvs_offset[0]]);\n                let uv0_3 = unpack2x16float(uvs.data[(*v3).uvs_offset[0]]);\n                \n                let uv1_1 = unpack2x16float(uvs.data[(*v1).uvs_offset[1]]);\n                let uv1_2 = unpack2x16float(uvs.data[(*v2).uvs_offset[1]]);\n                let uv1_3 = unpack2x16float(uvs.data[(*v3).uvs_offset[1]]);\n\n                let uv2_1 = unpack2x16float(uvs.data[(*v1).uvs_offset[2]]);\n                let uv2_2 = unpack2x16float(uvs.data[(*v2).uvs_offset[2]]);\n                let uv2_3 = unpack2x16float(uvs.data[(*v3).uvs_offset[2]]);\n                \n                let uv3_1 = unpack2x16float(uvs.data[(*v1).uvs_offset[3]]);\n                let uv3_2 = unpack2x16float(uvs.data[(*v2).uvs_offset[3]]);\n                let uv3_3 = unpack2x16float(uvs.data[(*v3).uvs_offset[3]]);\n\n                var uv_0 = interpolate_2d_attribute(uv0_1, uv0_2, uv0_3, deriv, delta);\n                var uv_1 = interpolate_2d_attribute(uv1_1, uv1_2, uv1_3, deriv, delta);\n                var uv_2 = interpolate_2d_attribute(uv2_1, uv2_2, uv2_3, deriv, delta);\n                var uv_3 = interpolate_2d_attribute(uv3_1, uv3_2, uv3_3, deriv, delta);\n                let uv_set = vec4<u32>(pack2x16float(uv_0.xy), pack2x16float(uv_1.xy), pack2x16float(uv_2.xy), pack2x16float(uv_3.xy));\n\n                let texture_color = sample_material_texture(material_id, TEXTURE_TYPE_BASE_COLOR, uv_set);\n                color = vec4<f32>(vertex_color.rgb * texture_color.rgb, alpha);\n\n                let n1 = decode_as_vec3(normals.data[(*v1).normal_offset]);\n                let n2 = decode_as_vec3(normals.data[(*v2).normal_offset]);\n                let n3 = decode_as_vec3(normals.data[(*v3).normal_offset]);\n\n                //let world_pos = barycentrics.x * p1 + barycentrics.y * p2 + barycentrics.z * p3;\n                //let n = barycentrics.x * n1 + barycentrics.y * n2 + barycentrics.z * n3;\n                let world_pos = interpolate_3d_attribute(p1.xyz, p2.xyz, p3.xyz, deriv, delta);\n                let n = interpolate_3d_attribute(n1, n2, n3, deriv, delta);\n\n                color = compute_brdf(world_pos.xyz, n, material_id, color, uv_set);\n            }\n\n            textureStore(render_target, pixel.xy, color);\n        }   \n    }\n}\n"}