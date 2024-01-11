{
  "spirv_code": [],
  "wgsl_code": "const MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;\nconst MAX_TEXTURE_COORDS_SET: u32 = 4u;\n\nconst TEXTURE_TYPE_BASE_COLOR: u32 = 0u;\nconst TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;\nconst TEXTURE_TYPE_NORMAL: u32 = 2u;\nconst TEXTURE_TYPE_EMISSIVE: u32 = 3u;\nconst TEXTURE_TYPE_OCCLUSION: u32 = 4u;\nconst TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;\nconst TEXTURE_TYPE_DIFFUSE: u32 = 6u;\nconst TEXTURE_TYPE_SPECULAR: u32 = 7u;\nconst TEXTURE_TYPE_SPECULAR_COLOR: u32 = 8u;\nconst TEXTURE_TYPE_TRANSMISSION: u32 = 9u;\nconst TEXTURE_TYPE_THICKNESS: u32 = 10u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_3: u32 = 11u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_4: u32 = 12u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_5: u32 = 13u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_6: u32 = 14u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_7: u32 = 15u;\nconst TEXTURE_TYPE_COUNT: u32 = 16u;\n\nconst MATERIAL_ALPHA_BLEND_OPAQUE = 0u;\nconst MATERIAL_ALPHA_BLEND_MASK = 1u;\nconst MATERIAL_ALPHA_BLEND_BLEND = 2u;\n\nconst MESH_FLAGS_NONE: u32 = 0u;\nconst MESH_FLAGS_VISIBLE: u32 = 1u;\nconst MESH_FLAGS_OPAQUE: u32 = 1u << 1u;\nconst MESH_FLAGS_TRANSPARENT: u32 = 1u << 2u;\nconst MESH_FLAGS_WIREFRAME: u32 = 1u << 3u;\nconst MESH_FLAGS_DEBUG: u32 = 1u << 4u;\nconst MESH_FLAGS_UI: u32 = 1u << 5u;\n\nconst CONSTANT_DATA_FLAGS_NONE: u32 = 0u;\nconst CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 1u << 1u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 1u << 2u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_CONE_AXIS: u32 = 1u << 3u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER: u32 = 1u << 4u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER: u32 = 1u << 5u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE: u32 = 1u << 6u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_NORMALS: u32 = 1u << 7u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_TANGENT: u32 = 1u << 8u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_BITANGENT: u32 = 1u << 9u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_BASE_COLOR: u32 = 1u << 10u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_METALLIC: u32 = 1u << 11u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_ROUGHNESS: u32 = 1u << 12u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_UV_0: u32 = 1u << 13u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_UV_1: u32 = 1u << 14u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_UV_2: u32 = 1u << 15u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_UV_3: u32 = 1u << 16u;\nconst CONSTANT_DATA_FLAGS_USE_IBL: u32 = 1u << 17u;\n\n\nconst MATH_PI: f32 = 3.14159265359;\nconst MATH_EPSILON = 0.0000001;\nconst MAX_FLOAT: f32 = 3.402823466e+38;\nconst MAX_TRACING_DISTANCE: f32 = 500.;\nconst HIT_EPSILON: f32 = 0.0001;\nconst INVALID_NODE: i32 = -1;\n\nconst VERTEX_ATTRIBUTE_HAS_POSITION: u32 = 0u;\nconst VERTEX_ATTRIBUTE_HAS_COLOR: u32 = 1u;\nconst VERTEX_ATTRIBUTE_HAS_NORMAL: u32 = 1u << 1u;\nconst VERTEX_ATTRIBUTE_HAS_TANGENT: u32 = 1u << 2u;\nconst VERTEX_ATTRIBUTE_HAS_UV1: u32 = 1u << 3u;\nconst VERTEX_ATTRIBUTE_HAS_UV2: u32 = 1u << 4u;\nconst VERTEX_ATTRIBUTE_HAS_UV3: u32 = 1u << 5u;\nconst VERTEX_ATTRIBUTE_HAS_UV4: u32 = 1u << 6u;\n\nconst MATERIAL_FLAGS_NONE: u32 = 0u;\nconst MATERIAL_FLAGS_UNLIT: u32 = 1u;\nconst MATERIAL_FLAGS_IRIDESCENCE: u32 = 1u << 1u;\nconst MATERIAL_FLAGS_ANISOTROPY: u32 = 1u << 2u;\nconst MATERIAL_FLAGS_CLEARCOAT: u32 = 1u << 3u;\nconst MATERIAL_FLAGS_SHEEN: u32 = 1u << 4u;\nconst MATERIAL_FLAGS_TRANSMISSION: u32 = 1u << 5u;\nconst MATERIAL_FLAGS_VOLUME: u32 = 1u << 6u;\nconst MATERIAL_FLAGS_EMISSIVE_STRENGTH: u32 = 1u << 7u;\nconst MATERIAL_FLAGS_METALLICROUGHNESS: u32 = 1u << 8u;\nconst MATERIAL_FLAGS_SPECULAR: u32 = 1u << 9u;\nconst MATERIAL_FLAGS_SPECULARGLOSSINESS: u32 = 1u << 10u;\nconst MATERIAL_FLAGS_IOR: u32 = 1u << 11u;\nconst MATERIAL_FLAGS_ALPHAMODE_OPAQUE: u32 = 1u << 12u;\nconst MATERIAL_FLAGS_ALPHAMODE_MASK: u32 = 1u << 13u;\nconst MATERIAL_FLAGS_ALPHAMODE_BLEND: u32 = 1u << 14u;\n\nconst LIGHT_TYPE_INVALID: u32 = 0u;\nconst LIGHT_TYPE_DIRECTIONAL: u32 = 1u;\nconst LIGHT_TYPE_POINT: u32 = 1u << 1u;\nconst LIGHT_TYPE_SPOT: u32 = 1u << 2u;\n\nstruct ConstantData {\n    view: mat4x4<f32>,\n    proj: mat4x4<f32>,\n    inverse_view_proj: mat4x4<f32>,\n    screen_width: f32,\n    screen_height: f32,\n    frame_index: u32,\n    flags: u32,\n    debug_uv_coords: vec2<f32>,\n    tlas_starting_index: u32,\n    indirect_light_num_bounces: u32\n};\n\nstruct RuntimeVertexData {\n    @location(0) world_pos: vec3<f32>,\n    @location(1) @interpolate(flat) mesh_index: u32,\n};\n\nstruct DrawCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_vertex: u32,\n    base_instance: u32,\n};\n\nstruct DrawIndexedCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_index: u32,\n    vertex_offset: i32,\n    base_instance: u32,\n};\n\nstruct DispatchCommandSize {\n    x: atomic<u32>,\n    y: atomic<u32>,\n    z: atomic<u32>,\n};\n\nstruct Mesh {\n    vertices_position_offset: u32,\n    vertices_attribute_offset: u32,\n    flags_and_vertices_attribute_layout: u32,\n    material_index: i32,\n    orientation: vec4<f32>,\n    position: vec3<f32>,\n    meshlets_offset: u32,\n    scale: vec3<f32>,\n    blas_index: u32,\n};\n\nstruct Meshlet {\n    @location(5) mesh_index: u32,\n    @location(6) indices_offset: u32,\n    @location(7) indices_count: u32,\n    @location(8) triangles_bhv_index: u32,\n    @location(9) center: vec3<f32>,\n    @location(10) cone_axis_cutoff: u32,\n};\n\nstruct BHVNode {\n    min: vec3<f32>,\n    miss: i32,\n    max: vec3<f32>,\n    reference: i32, //-1 or mesh_index or meshlet_index or triangle_index\n};\n\n\nstruct LightData {\n    position: vec3<f32>,\n    light_type: u32,\n    direction: vec3<f32>,\n    intensity: f32,\n    color: vec3<f32>,\n    range: f32,\n    inner_cone_angle: f32,\n    outer_cone_angle: f32,\n    _padding1: f32,\n    _padding2: f32,\n};\n\nstruct TextureData {\n    texture_index: u32,\n    layer_index: u32,\n    total_width: u32,\n    total_height: u32,\n    area: vec4<u32>,\n};\n\nstruct Material {\n    roughness_factor: f32,\n    metallic_factor: f32,\n    ior: f32,\n    transmission_factor: f32,\n    base_color: vec4<f32>,\n    emissive_color: vec3<f32>,\n    emissive_strength: f32,\n    diffuse_color: vec4<f32>,\n    specular_color: vec4<f32>,\n    specular_factors: vec4<f32>,\n    attenuation_color_and_distance: vec4<f32>,\n    thickness_factor: f32,\n    alpha_cutoff: f32,\n    occlusion_strength: f32,\n    flags: u32,\n    textures_index_and_coord_set: array<u32, TEXTURE_TYPE_COUNT>,\n};\n\n\nstruct Lights {\n    data: array<LightData>,\n};\n\nstruct Textures {\n    data: array<TextureData>,\n};\n\nstruct Materials {\n    data: array<Material>,\n};\n\nstruct DrawCommands {\n    data: array<DrawCommand>,\n};\n\nstruct DrawIndexedCommands {\n    data: array<DrawIndexedCommand>,\n};\n\nstruct Meshes {\n    data: array<Mesh>,\n};\n\nstruct Meshlets {\n    data: array<Meshlet>,\n};\n\nstruct Indices {\n    data: array<u32>,\n};\n\nstruct RuntimeVertices {\n    data: array<RuntimeVertexData>,\n};\n\nstruct VerticesPositions {\n    data: array<u32>,\n};\n\nstruct VerticesAttributes {\n    data: array<u32>,\n};\n\nstruct BHV {\n    data: array<BHVNode>,\n};\n\n\nstruct Ray {\n    origin: vec3<f32>,\n    t_min: f32,\n    direction: vec3<f32>,\n    t_max: f32,\n};\n\nstruct RadianceData {\n    origin: vec3<f32>,\n    seed_x: u32,\n    direction: vec3<f32>,\n    seed_y: u32,\n    radiance: vec3<f32>,\n    pixel: u32,\n    throughput_weight: vec3<f32>,\n    bounce: u32,\n};\n\nstruct RadianceDataBuffer {\n    data: array<RadianceData>,\n};\n\nstruct PixelData {\n    world_pos: vec3<f32>,\n    material_id: u32,\n    color: vec4<f32>,\n    normal: vec3<f32>,\n    mesh_id: u32, \n    tangent: vec4<f32>,\n    uv_set: array<vec2<f32>, 4>,\n};\n\nstruct TBN {\n    normal: vec3<f32>,\n    tangent: vec3<f32>,\n    binormal: vec3<f32>,\n}\n\nstruct MaterialInfo {\n    base_color: vec4<f32>,\n\n    f0: vec3<f32>,\n    ior: f32,\n\n    c_diff: vec3<f32>,\n    perceptual_roughness: f32,\n\n    metallic: f32,\n    specular_weight_and_anisotropy_strength: u32,\n    transmission_factor: f32,\n    thickness_factor: f32,\n\n    attenuation_color_and_distance: vec4<f32>,\n    sheen_color_and_roughness_factor: vec4<f32>,\n\n    clear_coat_f0: vec3<f32>,\n    clear_coat_factor: f32,\n\n    clear_coat_f90: vec3<f32>,\n    clear_coat_roughness_factor: f32,\n\n    clear_coat_normal: vec3<f32>,\n    iridescence_factor: f32,\n\n    anisotropicT: vec3<f32>,\n    iridescence_ior: f32,\n\n    anisotropicB: vec3<f32>,\n    iridescence_thickness: f32,\n\n    alpha_roughness: f32,\n    f90: vec3<f32>,\n    \n    f_color: vec4<f32>,\n    f_emissive: vec3<f32>,\n    f_diffuse: vec3<f32>,\n    f_diffuse_ibl: vec3<f32>,\n    f_specular: vec3<f32>,\n}\nfn quantize_unorm(v: f32, n: u32) -> u32 {\n    let scale = f32((1u << n) - 1u);\n    return u32(0.5 + (v * scale));\n}\nfn quantize_snorm(v: f32, n: u32) -> u32 {\n    let c = (1u << (n - 1u)) - 1u;\n    let scale = f32(c);\n    if v < 0. {\n        return (u32(-v * scale) & c) | (1u << (n - 1u));\n    } else {\n        return u32(v * scale) & c;\n    }\n}\n\nfn decode_unorm(i: u32, n: u32) -> f32 {    \n    let scale = f32((1u << n) - 1u);\n    if (i == 0u) {\n        return 0.;\n    } else if (i == u32(scale)) {\n        return 1.;\n    } else {\n        return (f32(i) - 0.5) / scale;\n    }\n}\n\nfn decode_snorm(i: u32, n: u32) -> f32 {\n    let s = i >> (n - 1u);\n    let c = (1u << (n - 1u)) - 1u;\n    let scale = f32(c);\n    if s > 0u {\n        let r = f32(i & c) / scale;\n        return -r;\n    } else {\n        return f32(i & c) / scale;\n    }\n}\n\nfn pack_3_f32_to_unorm(value: vec3<f32>) -> u32 {\n    let x = quantize_unorm(value.x, 10u) << 20u;\n    let y = quantize_unorm(value.y, 10u) << 10u;\n    let z = quantize_unorm(value.z, 10u);\n    return (x | y | z);\n}\nfn unpack_unorm_to_3_f32(v: u32) -> vec3<f32> {\n    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_unorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn pack_3_f32_to_snorm(value: vec3<f32>) -> u32 {\n    let x = quantize_snorm(value.x, 10u) << 20u;\n    let y = quantize_snorm(value.y, 10u) << 10u;\n    let z = quantize_snorm(value.z, 10u);\n    return (x | y | z);\n}\nfn unpack_snorm_to_3_f32(v: u32) -> vec3<f32> {\n    let vx = decode_snorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_snorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_snorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn pack_normal(normal: vec3<f32>) -> vec2<f32> {\n    return vec2<f32>(normal.xy * 0.5 + 0.5);\n}\nfn unpack_normal(uv: vec2<f32>) -> vec3<f32> {\n    return vec3<f32>(uv.xy * 2. - 1., sqrt(1.-dot(uv.xy, uv.xy)));\n}\n\nfn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {\n    let r = quantize_unorm(value.x, 8u) << 24u;\n    let g = quantize_unorm(value.y, 8u) << 16u;\n    let b = quantize_unorm(value.z, 8u) << 8u;\n    let a = quantize_unorm(value.w, 8u);\n    return (r | g | b | a);\n}\nfn unpack_snorm_to_4_f32(v: u32) -> vec4<f32> {\n    let r = decode_snorm((v >> 24u) & 255u, 8u);\n    let g = decode_snorm((v >> 16u) & 255u, 8u);\n    let b = decode_snorm((v >> 8u) & 255u, 8u);\n    let a = decode_snorm(v & 255u, 8u);\n    return vec4<f32>(r,g,b,a);\n}\nfn unpack_unorm_to_4_f32(v: u32) -> vec4<f32> {\n    let r = decode_unorm((v >> 24u) & 255u, 8u);\n    let g = decode_unorm((v >> 16u) & 255u, 8u);\n    let b = decode_unorm((v >> 8u) & 255u, 8u);\n    let a = decode_unorm(v & 255u, 8u);\n    return vec4<f32>(r,g,b,a);\n}\n\nfn iq_hash(v: vec2<f32>) -> f32 {\n    return fract(sin(dot(v, vec2(11.9898, 78.233))) * 43758.5453);\n}\nfn blue_noise(in: vec2<f32>) -> f32 {\n    var v =  iq_hash( in + vec2<f32>(-1., 0.) )\n             + iq_hash( in + vec2<f32>( 1., 0.) )\n             + iq_hash( in + vec2<f32>( 0., 1.) )\n             + iq_hash( in + vec2<f32>( 0.,-1.) ); \n    v /= 4.;\n    return (iq_hash(in) - v + .5);\n}\n\n// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.\nfn hash( x: u32 ) -> u32 {\n    var v = x;\n    v += ( v << 10u );\n    v ^= ( v >>  6u );\n    v += ( v <<  3u );\n    v ^= ( v >> 11u );\n    v += ( v << 15u );\n    return v;\n}\n\nfn hash1(seed: f32) -> f32 {\n    var p = fract(seed * .1031);\n    p *= p + 33.33;\n    p *= p + p;\n    return fract(p);\n}\n\nfn hash2(seed: ptr<function, f32>) -> vec2<f32> {\n    let a = (*seed) + 0.1;\n    let b = a + 0.1;\n    (*seed) = b;\n    return fract(sin(vec2(a,b))*vec2(43758.5453123,22578.1459123));\n}\n\nfn hash3(seed: ptr<function, f32>) -> vec3<f32> {\n    let a = (*seed) + 0.1;\n    let b = a + 0.1;\n    let c = b + 0.1;\n    (*seed) = c;\n    return fract(sin(vec3(a,b,c))*vec3(43758.5453123,22578.1459123,19642.3490423));\n}\n\n// This is PCG2d\nfn get_random_numbers(seed: ptr<function, vec2<u32>>) -> vec2<f32> {\n    var v = (*seed) * 1664525u + 1013904223u;\n    v.x += v.y * 1664525u; v.y += v.x * 1664525u;\n    v ^= v >> vec2u(16u);\n    v.x += v.y * 1664525u; v.y += v.x * 1664525u;\n    v ^= v >> vec2u(16u);\n    *seed = v;\n    return vec2<f32>(v) * 2.32830643654e-10;\n}\n\nfn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) \n{\n    let c = *ptr_a;\n    *ptr_a = *ptr_b;\n    *ptr_b = c;\n}\n\nfn mod_f32(v: f32, m: f32) -> f32\n{\n    return v - (m * floor(v/m));\n}\n\nfn clamped_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {\n    return clamp(dot(a,b), 0., 1.);\n}\n\nfn has_vertex_attribute(vertex_attribute_layout: u32, attribute_to_check: u32) -> bool {\n    return bool(vertex_attribute_layout & attribute_to_check);\n}\nfn vertex_attribute_offset(vertex_attribute_layout: u32, attribute_to_check: u32) -> i32 \n{\n    if(has_vertex_attribute(vertex_attribute_layout, attribute_to_check)) {\n        let mask = (vertex_attribute_layout & 0x0000FFFFu) & (~attribute_to_check & (attribute_to_check - 1u));\n        return i32(countOneBits(mask));\n    }\n    return -1;\n}\nfn vertex_layout_stride(vertex_attribute_layout: u32) -> u32 \n{\n    return countOneBits((vertex_attribute_layout & 0x0000FFFFu));\n}\n\nstruct DebugVertex {\n    @builtin(vertex_index) index: u32,\n    @location(0) position: vec3<f32>,\n    @location(1) color: u32,\n};\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) color: vec4<f32>,\n};\n\nstruct FragmentOutput {\n    @location(0) color: vec4<f32>,\n};\n\n\n@group(0) @binding(0)\nvar<uniform> constant_data: ConstantData;\n\n\nfn extract_scale(m: mat4x4<f32>) -> vec3<f32> \n{\n    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);\n    let sx = length(s[0]);\n    let sy = length(s[1]);\n    let det = determinant(s);\n    var sz = length(s[2]);\n    if (det < 0.) \n    {\n        sz = -sz;\n    }\n    return vec3<f32>(sx, sy, sz);\n}\n\nfn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> \n{\n    if (row == 1u) {\n        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);\n    } else if (row == 2u) {\n        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);\n    } else if (row == 3u) {\n        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);\n    } else {        \n        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);\n    }\n}\n\nfn normalize_plane(plane: vec4<f32>) -> vec4<f32> \n{\n    return (plane / length(plane.xyz));\n}\n\nfn rotate_vector(v: vec3<f32>, orientation: vec4<f32>) -> vec3<f32> \n{\n    return v + 2. * cross(orientation.xyz, cross(orientation.xyz, v) + orientation.w * v);\n}\n\nfn transform_vector(v: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> vec3<f32> \n{\n    return rotate_vector(v, orientation) * scale + position;\n}\n\nfn matrix_from_translation(translation: vec3<f32>) -> mat4x4<f32> {\n    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0),\n                      vec4<f32>(0.0, 1.0, 0.0, 0.0),\n                      vec4<f32>(0.0, 0.0, 1.0, 0.0),\n                      vec4<f32>(translation.x, translation.y, translation.z, 1.0));\n}\n\nfn matrix_from_scale(scale: vec3<f32>) -> mat4x4<f32> {\n    return mat4x4<f32>(vec4<f32>(scale.x, 0.0, 0.0, 0.0),\n                      vec4<f32>(0.0, scale.y, 0.0, 0.0),\n                      vec4<f32>(0.0, 0.0, scale.z, 0.0),\n                      vec4<f32>(0.0, 0.0, 0.0, 1.0));\n}\n\nfn matrix_from_orientation(q: vec4<f32>) -> mat4x4<f32> {\n    let xx = q.x * q.x;\n    let yy = q.y * q.y;\n    let zz = q.z * q.z;\n    let xy = q.x * q.y;\n    let xz = q.x * q.z;\n    let yz = q.y * q.z;\n    let wx = q.w * q.x;\n    let wy = q.w * q.y;\n    let wz = q.w * q.z;\n\n    let m00 = 1.0 - 2.0 * (yy + zz);\n    let m01 = 2.0 * (xy - wz);\n    let m02 = 2.0 * (xz + wy);\n\n    let m10 = 2.0 * (xy + wz);\n    let m11 = 1.0 - 2.0 * (xx + zz);\n    let m12 = 2.0 * (yz - wx);\n\n    let m20 = 2.0 * (xz - wy);\n    let m21 = 2.0 * (yz + wx);\n    let m22 = 1.0 - 2.0 * (xx + yy);\n\n    // Utilizza la funzione mat4x4 per creare la matrice 4x4\n    return mat4x4<f32>(\n        vec4<f32>(m00, m01, m02, 0.0),\n        vec4<f32>(m10, m11, m12, 0.0),\n        vec4<f32>(m20, m21, m22, 0.0),\n        vec4<f32>(0.0, 0.0, 0.0, 1.0)\n    );\n}\n\nfn transform_matrix(position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> mat4x4<f32> {\n    let translation_matrix = matrix_from_translation(position);\n    let rotation_matrix = matrix_from_orientation(orientation);\n    let scale_matrix = matrix_from_scale(scale);    \n    return translation_matrix * rotation_matrix * scale_matrix;\n}\n\nfn matrix_inverse(m: mat4x4<f32>) -> mat4x4<f32> {\n    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];\n    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];\n    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];\n    let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];\n\n    let b00 = a00 * a11 - a01 * a10;\n    let b01 = a00 * a12 - a02 * a10;\n    let b02 = a00 * a13 - a03 * a10;\n    let b03 = a01 * a12 - a02 * a11;\n    let b04 = a01 * a13 - a03 * a11;\n    let b05 = a02 * a13 - a03 * a12;\n    let b06 = a20 * a31 - a21 * a30;\n    let b07 = a20 * a32 - a22 * a30;\n    let b08 = a20 * a33 - a23 * a30;\n    let b09 = a21 * a32 - a22 * a31;\n    let b10 = a21 * a33 - a23 * a31;\n    let b11 = a22 * a33 - a23 * a32;\n\n    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;\n    \n    // Ottimizzazione: Calcola l'inverso del determinante una sola volta\n    let invDet = 1.0 / det;\n\n    return mat4x4<f32>(\n        vec4<f32>((a11 * b11 - a12 * b10 + a13 * b09) * invDet, (a02 * b10 - a01 * b11 - a03 * b09) * invDet, (a31 * b05 - a32 * b04 + a33 * b03) * invDet, (a22 * b04 - a21 * b05 - a23 * b03) * invDet),\n        vec4<f32>((a12 * b08 - a10 * b11 - a13 * b07) * invDet, (a00 * b11 - a02 * b08 + a03 * b07) * invDet, (a32 * b02 - a30 * b05 - a33 * b01) * invDet, (a20 * b05 - a22 * b02 + a23 * b01) * invDet),\n        vec4<f32>((a10 * b10 - a11 * b08 + a13 * b06) * invDet, (a01 * b08 - a00 * b10 - a03 * b06) * invDet, (a30 * b04 - a31 * b02 + a33 * b00) * invDet, (a21 * b02 - a20 * b04 - a23 * b00) * invDet),\n        vec4<f32>((a11 * b07 - a10 * b09 - a12 * b06) * invDet, (a00 * b09 - a01 * b07 + a02 * b06) * invDet, (a31 * b01 - a30 * b03 - a32 * b00) * invDet, (a20 * b03 - a21 * b01 + a22 * b00) * invDet)\n    );\n}\n\n@vertex\nfn vs_main(\n    v_in: DebugVertex,\n) -> VertexOutput {\n\n    var vertex_out: VertexOutput;\n    vertex_out.clip_position = constant_data.proj * constant_data.view * vec4<f32>(v_in.position, 1.);\n    vertex_out.color = unpack_unorm_to_4_f32(v_in.color);\n\n    return vertex_out;\n}\n\n@fragment\nfn fs_main(\n    @builtin(primitive_index) primitive_index: u32,\n    v_in: VertexOutput,\n) -> FragmentOutput {    \n    var fragment_out: FragmentOutput;\n    fragment_out.color = v_in.color;\n    return fragment_out;\n}\n"
}