{
  "spirv_code": [],
  "wgsl_code": "const DEFAULT_WIDTH: u32 = 1920u;\nconst DEFAULT_HEIGHT: u32 = 1080u;\nconst SIZE_OF_DATA_BUFFER_ELEMENT: u32 = 4u;\nconst MAX_LOD_LEVELS: u32 = 8u;\nconst MAX_NUM_LIGHTS: u32 = 1024u;\nconst MAX_NUM_TEXTURES: u32 = 65536u;\nconst MAX_NUM_MATERIALS: u32 = 65536u;\n\nconst CONSTANT_DATA_FLAGS_NONE: u32 = 0u;\nconst CONSTANT_DATA_FLAGS_USE_IBL: u32 = 1u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 1u << 1u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 1u << 2u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER: u32 = 1u << 3u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER: u32 = 1u << 4u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE: u32 = 1u << 5u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_NORMALS: u32 = 1u << 6u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_TANGENT: u32 = 1u << 7u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_BITANGENT: u32 = 1u << 8u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_BASE_COLOR: u32 = 1u << 9u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_METALLIC: u32 = 1u << 10u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_ROUGHNESS: u32 = 1u << 11u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_UV_0: u32 = 1u << 12u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_UV_1: u32 = 1u << 13u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_UV_2: u32 = 1u << 14u;\nconst CONSTANT_DATA_FLAGS_DISPLAY_UV_3: u32 = 1u << 15u;\n\nconst MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;\nconst MAX_TEXTURE_COORDS_SET: u32 = 4u;\n\nconst TEXTURE_TYPE_BASE_COLOR: u32 = 0u;\nconst TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;\nconst TEXTURE_TYPE_NORMAL: u32 = 2u;\nconst TEXTURE_TYPE_EMISSIVE: u32 = 3u;\nconst TEXTURE_TYPE_OCCLUSION: u32 = 4u;\nconst TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;\nconst TEXTURE_TYPE_DIFFUSE: u32 = 6u;\nconst TEXTURE_TYPE_SPECULAR: u32 = 7u;\nconst TEXTURE_TYPE_SPECULAR_COLOR: u32 = 8u;\nconst TEXTURE_TYPE_TRANSMISSION: u32 = 9u;\nconst TEXTURE_TYPE_THICKNESS: u32 = 10u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_3: u32 = 11u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_4: u32 = 12u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_5: u32 = 13u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_6: u32 = 14u;\nconst TEXTURE_TYPE_EMPTY_FOR_PADDING_7: u32 = 15u;\nconst TEXTURE_TYPE_COUNT: u32 = 16u;\n\nconst MATERIAL_ALPHA_BLEND_OPAQUE = 0u;\nconst MATERIAL_ALPHA_BLEND_MASK = 1u;\nconst MATERIAL_ALPHA_BLEND_BLEND = 2u;\n\nconst MESH_FLAGS_NONE: u32 = 0u;\nconst MESH_FLAGS_VISIBLE: u32 = 1u;\nconst MESH_FLAGS_OPAQUE: u32 = 1u << 1u;\nconst MESH_FLAGS_TRANSPARENT: u32 = 1u << 2u;\nconst MESH_FLAGS_WIREFRAME: u32 = 1u << 3u;\nconst MESH_FLAGS_DEBUG: u32 = 1u << 4u;\nconst MESH_FLAGS_UI: u32 = 1u << 5u;\n\n\nconst MATH_PI: f32 = 3.14159265359;\nconst MATH_EPSILON = 0.0000001;\nconst MAX_FLOAT: f32 = 3.402823466e+38;\nconst MAX_TRACING_DISTANCE: f32 = 500.;\nconst HIT_EPSILON: f32 = 0.0001;\nconst INVALID_NODE: i32 = -1;\n\nconst VERTEX_ATTRIBUTE_HAS_POSITION: u32 = 0u;\nconst VERTEX_ATTRIBUTE_HAS_COLOR: u32 = 1u;\nconst VERTEX_ATTRIBUTE_HAS_NORMAL: u32 = 1u << 1u;\nconst VERTEX_ATTRIBUTE_HAS_TANGENT: u32 = 1u << 2u;\nconst VERTEX_ATTRIBUTE_HAS_UV1: u32 = 1u << 3u;\nconst VERTEX_ATTRIBUTE_HAS_UV2: u32 = 1u << 4u;\nconst VERTEX_ATTRIBUTE_HAS_UV3: u32 = 1u << 5u;\nconst VERTEX_ATTRIBUTE_HAS_UV4: u32 = 1u << 6u;\n\nconst MATERIAL_FLAGS_NONE: u32 = 0u;\nconst MATERIAL_FLAGS_UNLIT: u32 = 1u;\nconst MATERIAL_FLAGS_IRIDESCENCE: u32 = 1u << 1u;\nconst MATERIAL_FLAGS_ANISOTROPY: u32 = 1u << 2u;\nconst MATERIAL_FLAGS_CLEARCOAT: u32 = 1u << 3u;\nconst MATERIAL_FLAGS_SHEEN: u32 = 1u << 4u;\nconst MATERIAL_FLAGS_TRANSMISSION: u32 = 1u << 5u;\nconst MATERIAL_FLAGS_VOLUME: u32 = 1u << 6u;\nconst MATERIAL_FLAGS_EMISSIVE_STRENGTH: u32 = 1u << 7u;\nconst MATERIAL_FLAGS_METALLICROUGHNESS: u32 = 1u << 8u;\nconst MATERIAL_FLAGS_SPECULAR: u32 = 1u << 9u;\nconst MATERIAL_FLAGS_SPECULARGLOSSINESS: u32 = 1u << 10u;\nconst MATERIAL_FLAGS_IOR: u32 = 1u << 11u;\nconst MATERIAL_FLAGS_ALPHAMODE_OPAQUE: u32 = 1u << 12u;\nconst MATERIAL_FLAGS_ALPHAMODE_MASK: u32 = 1u << 13u;\nconst MATERIAL_FLAGS_ALPHAMODE_BLEND: u32 = 1u << 14u;\n\nconst LIGHT_TYPE_INVALID: u32 = 0u;\nconst LIGHT_TYPE_DIRECTIONAL: u32 = 1u;\nconst LIGHT_TYPE_POINT: u32 = 1u << 1u;\nconst LIGHT_TYPE_SPOT: u32 = 1u << 2u;\n\nstruct ConstantData {\n    view: mat4x4<f32>,\n    inv_view: mat4x4<f32>,\n    proj: mat4x4<f32>,\n    view_proj: mat4x4<f32>,\n    inverse_view_proj: mat4x4<f32>,\n    screen_width: f32,\n    screen_height: f32,\n    frame_index: u32,\n    flags: u32,\n    debug_uv_coords: vec2<f32>,\n    tlas_starting_index: u32,\n    indirect_light_num_bounces: u32,\n    lut_pbr_charlie_texture_index: u32,\n    lut_pbr_ggx_texture_index: u32,\n    environment_map_texture_index: u32,\n    num_lights: u32,\n    forced_lod_level: i32,\n    _empty1: u32,\n    _empty2: u32,\n    _empty3: u32,\n};\n\nstruct RuntimeVertexData {\n    @location(0) world_pos: vec3<f32>,\n    @location(1) @interpolate(flat) mesh_index: u32,\n};\n\nstruct DrawCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_vertex: u32,\n    base_instance: u32,\n};\n\nstruct DrawIndexedCommand {\n    vertex_count: u32,\n    instance_count: u32,\n    base_index: u32,\n    vertex_offset: i32,\n    base_instance: u32,\n};\n\nstruct DispatchCommandSize {\n    x: atomic<u32>,\n    y: atomic<u32>,\n    z: atomic<u32>,\n};\n\nstruct Mesh {\n    vertices_position_offset: u32,\n    vertices_attribute_offset: u32,\n    flags_and_vertices_attribute_layout: u32,\n    material_index: i32,\n    orientation: vec4<f32>,\n    position: vec3<f32>,\n    meshlets_offset: u32,\n    scale: vec3<f32>,\n    blas_index: u32,\n    lods_meshlets_offset: array<u32, MAX_LOD_LEVELS>,\n};\n\nstruct Meshlet {\n    @location(5) mesh_index_and_lod_level: u32, // 29 mesh + 3 lod bits\n    @location(6) indices_offset: u32,\n    @location(7) indices_count: u32,\n    @location(8) bvh_offset: u32,\n    @location(9) child_meshlets: vec4<i32>,\n};\n\nstruct BHVNode {\n    min: vec3<f32>,\n    miss: i32,\n    max: vec3<f32>,\n    reference: i32, //-1 or mesh_index or meshlet_index or triangle_index\n};\n\n\nstruct LightData {\n    position: vec3<f32>,\n    light_type: u32,\n    direction: vec3<f32>,\n    intensity: f32,\n    color: vec3<f32>,\n    range: f32,\n    inner_cone_angle: f32,\n    outer_cone_angle: f32,\n    _padding1: f32,\n    _padding2: f32,\n};\n\nstruct TextureData {\n    texture_and_layer_index: i32,\n    min: u32,\n    max: u32,\n    size: u32,\n};\n\nstruct Material {\n    roughness_factor: f32,\n    metallic_factor: f32,\n    ior: f32,\n    transmission_factor: f32,\n    base_color: vec4<f32>,\n    emissive_color: vec3<f32>,\n    emissive_strength: f32,\n    diffuse_color: vec4<f32>,\n    specular_color: vec4<f32>,\n    specular_factors: vec4<f32>,\n    attenuation_color_and_distance: vec4<f32>,\n    thickness_factor: f32,\n    normal_scale_and_alpha_cutoff: u32,\n    occlusion_strength: f32,\n    flags: u32,\n    textures_index_and_coord_set: array<u32, TEXTURE_TYPE_COUNT>,\n};\n\n\nstruct Lights {\n    data: array<LightData, MAX_NUM_LIGHTS>,\n};\n\nstruct Textures {\n    data: array<TextureData>,\n};\n\nstruct Materials {\n    data: array<Material>,\n};\n\nstruct DrawCommands {\n    data: array<DrawCommand>,\n};\n\nstruct DrawIndexedCommands {\n    data: array<DrawIndexedCommand>,\n};\n\nstruct Meshes {\n    data: array<Mesh>,\n};\n\nstruct Meshlets {\n    data: array<Meshlet>,\n};\n\nstruct Indices {\n    data: array<u32>,\n};\n\nstruct RuntimeVertices {\n    data: array<RuntimeVertexData>,\n};\n\nstruct VerticesPositions {\n    data: array<u32>,\n};\n\nstruct VerticesAttributes {\n    data: array<u32>,\n};\n\nstruct BHV {\n    data: array<BHVNode>,\n};\n\n\nstruct Ray {\n    origin: vec3<f32>,\n    t_min: f32,\n    direction: vec3<f32>,\n    t_max: f32,\n};\n\nstruct PixelData {\n    world_pos: vec3<f32>,\n    material_id: u32,\n    color: vec4<f32>,\n    normal: vec3<f32>,\n    mesh_id: u32, \n    tangent: vec4<f32>,\n    uv_set: array<vec4<f32>, 4>,\n};\n\nstruct TBN {\n    normal: vec3<f32>,\n    tangent: vec3<f32>,\n    binormal: vec3<f32>,\n};\n\nstruct MaterialInfo {\n    base_color: vec4<f32>,\n\n    f0: vec3<f32>,\n    ior: f32,\n\n    c_diff: vec3<f32>,\n    perceptual_roughness: f32,\n\n    metallic: f32,\n    specular_weight_and_anisotropy_strength: u32,\n    transmission_factor: f32,\n    thickness_factor: f32,\n\n    attenuation_color_and_distance: vec4<f32>,\n    sheen_color_and_roughness_factor: vec4<f32>,\n\n    clear_coat_f0: vec3<f32>,\n    clear_coat_factor: f32,\n\n    clear_coat_f90: vec3<f32>,\n    clear_coat_roughness_factor: f32,\n\n    clear_coat_normal: vec3<f32>,\n    iridescence_factor: f32,\n\n    anisotropicT: vec3<f32>,\n    iridescence_ior: f32,\n\n    anisotropicB: vec3<f32>,\n    iridescence_thickness: f32,\n\n    alpha_roughness: f32,\n    f90: vec3<f32>,\n    \n    f_color: vec4<f32>,\n    f_emissive: vec3<f32>,\n    f_diffuse: vec3<f32>,\n    f_diffuse_ibl: vec3<f32>,\n    f_specular: vec3<f32>,\n};\nfn quantize_unorm(v: f32, n: u32) -> u32 {\n    let scale = f32((1u << n) - 1u);\n    return u32(0.5 + (v * scale));\n}\nfn quantize_snorm(v: f32, n: u32) -> u32 {\n    let c = (1u << (n - 1u)) - 1u;\n    let scale = f32(c);\n    if v < 0. {\n        return (u32(-v * scale) & c) | (1u << (n - 1u));\n    } else {\n        return u32(v * scale) & c;\n    }\n}\n\nfn decode_unorm(i: u32, n: u32) -> f32 {    \n    let scale = f32((1u << n) - 1u);\n    if (i == 0u) {\n        return 0.;\n    } else if (i == u32(scale)) {\n        return 1.;\n    } else {\n        return (f32(i) - 0.5) / scale;\n    }\n}\n\nfn decode_snorm(i: u32, n: u32) -> f32 {\n    let s = i >> (n - 1u);\n    let c = (1u << (n - 1u)) - 1u;\n    let scale = f32(c);\n    if s > 0u {\n        let r = f32(i & c) / scale;\n        return -r;\n    } else {\n        return f32(i & c) / scale;\n    }\n}\n\nfn pack_3_f32_to_unorm(value: vec3<f32>) -> u32 {\n    let x = quantize_unorm(value.x, 10u) << 20u;\n    let y = quantize_unorm(value.y, 10u) << 10u;\n    let z = quantize_unorm(value.z, 10u);\n    return (x | y | z);\n}\nfn unpack_unorm_to_3_f32(v: u32) -> vec3<f32> {\n    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_unorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn pack_3_f32_to_snorm(value: vec3<f32>) -> u32 {\n    let x = quantize_snorm(value.x, 10u) << 20u;\n    let y = quantize_snorm(value.y, 10u) << 10u;\n    let z = quantize_snorm(value.z, 10u);\n    return (x | y | z);\n}\nfn unpack_snorm_to_3_f32(v: u32) -> vec3<f32> {\n    let vx = decode_snorm((v >> 20u) & 0x000003FFu, 10u);\n    let vy = decode_snorm((v >> 10u) & 0x000003FFu, 10u);\n    let vz = decode_snorm(v & 0x000003FFu, 10u);\n    return vec3<f32>(vx, vy, vz);\n}\n\nfn unpack_normal(f: f32) -> vec3<f32> {\n\tvar f_var = f;\n\tvar flipZ: f32 = sign(f_var);\n\tf_var = abs(f_var);\n\tlet atanXY: f32 = floor(f_var) / 67.5501 * (3.1415927 * 2.) - 3.1415927;\n\tvar n: vec3<f32> = vec3<f32>(sin(atanXY), cos(atanXY), 0.);\n\tn.z = fract(f_var) * 1869.2296 / 427.67993;\n\tn = normalize(n);\n\tn.z = n.z * (flipZ);\n\treturn n;\n} \n\nfn pack_normal(n: vec3<f32>) -> f32 {\n\tvar n_var = n;\n\tlet flipZ: f32 = sign(n_var.z);\n\tn_var.z = abs(n_var.z);\n\tn_var = n_var / (23.065746);\n\tlet xy: f32 = floor((atan2(n_var.x, n_var.y) + 3.1415927) / (3.1415927 * 2.) * 67.5501);\n\tvar z: f32 = floor(n_var.z * 427.67993) / 1869.2296;\n\tz = z * (1. / max(0.01, length(vec2<f32>(n_var.x, n_var.y))));\n\treturn (xy + z) * flipZ;\n} \n\n\nfn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {\n    let r = quantize_unorm(value.x, 8u) << 24u;\n    let g = quantize_unorm(value.y, 8u) << 16u;\n    let b = quantize_unorm(value.z, 8u) << 8u;\n    let a = quantize_unorm(value.w, 8u);\n    return (r | g | b | a);\n}\nfn unpack_snorm_to_4_f32(v: u32) -> vec4<f32> {\n    let r = decode_snorm((v >> 24u) & 255u, 8u);\n    let g = decode_snorm((v >> 16u) & 255u, 8u);\n    let b = decode_snorm((v >> 8u) & 255u, 8u);\n    let a = decode_snorm(v & 255u, 8u);\n    return vec4<f32>(r,g,b,a);\n}\nfn unpack_unorm_to_4_f32(v: u32) -> vec4<f32> {\n    let r = decode_unorm((v >> 24u) & 255u, 8u);\n    let g = decode_unorm((v >> 16u) & 255u, 8u);\n    let b = decode_unorm((v >> 8u) & 255u, 8u);\n    let a = decode_unorm(v & 255u, 8u);\n    return vec4<f32>(r,g,b,a);\n}\n\nfn sign_not_zero(v: vec2<f32>) -> vec2<f32> {\n\treturn vec2<f32>(select(-1., 1., v.x >= 0.), select(-1., 1., v.y >= 0.));\n} \n\nfn octahedral_mapping(v: vec3<f32>) -> vec2<f32> {\n\tlet l1norm: f32 = abs(v.x) + abs(v.y) + abs(v.z);\n\tvar result: vec2<f32> = v.xy * (1. / l1norm);\n\tif (v.z < 0.) {\n\t\tresult = (1. - abs(result.yx)) * sign_not_zero(result.xy);\n\t}\n\treturn result;\n} \n\nfn octahedral_unmapping(o: vec2<f32>) -> vec3<f32> {\n\tvar v: vec3<f32> = vec3<f32>(o.x, o.y, 1. - abs(o.x) - abs(o.y));\n\tif (v.z < 0.) {\n\t\tvar vxy = v.xy;\n        vxy = (1. - abs(v.yx)) * sign_not_zero(v.xy);\n        v.x = vxy.x;\n        v.y = vxy.y;\n\t}\n\treturn normalize(v);\n} \n\nfn f32tof16(v: f32) -> u32 {\n    return pack2x16float(vec2<f32>(v, 0.));\n}\n\nfn f16tof32(v: u32) -> f32 {\n    return unpack2x16float(v & 0x0000FFFFu).x;\n}\n\nfn pack_into_R11G11B10F(rgb: vec3<f32>) -> u32 {\n\tlet r = (f32tof16(rgb.r) << 17u) & 0xFFE00000u;\n\tlet g = (f32tof16(rgb.g) << 6u) & 0x001FFC00u;\n\tlet b = (f32tof16(rgb.b) >> 5u) & 0x000003FFu;\n\treturn r | g | b;\n} \n\nfn unpack_from_R11G11B10F(rgb: u32) -> vec3<f32> {\n\tlet r = f16tof32((rgb >> 17u) & 0x7FF0u);\n\tlet g = f16tof32((rgb >> 6u) & 0x7FF0u);\n\tlet b = f16tof32((rgb << 5u) & 0x7FE0u);\n\treturn vec3<f32>(r, g, b);\n} \n\n\nfn iq_hash(v: vec2<f32>) -> f32 {\n    return fract(sin(dot(v, vec2(11.9898, 78.233))) * 43758.5453);\n}\nfn blue_noise(in: vec2<f32>) -> f32 {\n    var v =  iq_hash( in + vec2<f32>(-1., 0.) )\n             + iq_hash( in + vec2<f32>( 1., 0.) )\n             + iq_hash( in + vec2<f32>( 0., 1.) )\n             + iq_hash( in + vec2<f32>( 0.,-1.) ); \n    v /= 4.;\n    return (iq_hash(in) - v + .5);\n}\n\n// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.\nfn hash( x: u32 ) -> u32 {\n    var v = x;\n    v += ( v << 10u );\n    v ^= ( v >>  6u );\n    v += ( v <<  3u );\n    v ^= ( v >> 11u );\n    v += ( v << 15u );\n    return v;\n}\n\nfn hash1(seed: f32) -> f32 {\n    var p = fract(seed * .1031);\n    p *= p + 33.33;\n    p *= p + p;\n    return fract(p);\n}\n\nfn hash2(seed: ptr<function, f32>) -> vec2<f32> {\n    let a = (*seed) + 0.1;\n    let b = a + 0.1;\n    (*seed) = b;\n    return fract(sin(vec2(a,b))*vec2(43758.5453123,22578.1459123));\n}\n\nfn hash3(seed: ptr<function, f32>) -> vec3<f32> {\n    let a = (*seed) + 0.1;\n    let b = a + 0.1;\n    let c = b + 0.1;\n    (*seed) = c;\n    return fract(sin(vec3(a,b,c))*vec3(43758.5453123,22578.1459123,19642.3490423));\n}\n\n// This is PCG2d\nfn get_random_numbers(seed: ptr<function, vec2<u32>>) -> vec2<f32> {\n    var v = (*seed) * 1664525u + 1013904223u;\n    v.x += v.y * 1664525u; v.y += v.x * 1664525u;\n    v ^= v >> vec2u(16u);\n    v.x += v.y * 1664525u; v.y += v.x * 1664525u;\n    v ^= v >> vec2u(16u);\n    *seed = v;\n    return vec2<f32>(v) * 2.32830643654e-10;\n}\n\nfn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) \n{\n    let c = *ptr_a;\n    *ptr_a = *ptr_b;\n    *ptr_b = c;\n}\n\nfn mod_f32(v: f32, m: f32) -> f32\n{\n    return v - (m * floor(v/m));\n}\n\nfn clamped_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {\n    return clamp(dot(a,b), 0., 1.);\n}\n\nfn has_vertex_attribute(vertex_attribute_layout: u32, attribute_to_check: u32) -> bool {\n    return bool(vertex_attribute_layout & attribute_to_check);\n}\nfn vertex_attribute_offset(vertex_attribute_layout: u32, attribute_to_check: u32) -> i32 \n{\n    if(has_vertex_attribute(vertex_attribute_layout, attribute_to_check)) {\n        let mask = (vertex_attribute_layout & 0x0000FFFFu) & (~attribute_to_check & (attribute_to_check - 1u));\n        return i32(countOneBits(mask));\n    }\n    return -1;\n}\nfn vertex_layout_stride(vertex_attribute_layout: u32) -> u32 \n{\n    return countOneBits((vertex_attribute_layout & 0x0000FFFFu));\n}\n\nstruct CullingData {\n    view: mat4x4<f32>,\n    mesh_flags: u32,\n    lod0_meshlets_count: u32,\n    _padding1: u32,\n    _padding2: u32,\n};\n\n@group(0) @binding(0)\nvar<uniform> constant_data: ConstantData;\n@group(0) @binding(1)\nvar<uniform> culling_data: CullingData;\n@group(0) @binding(2)\nvar<storage, read> meshlets: Meshlets;\n@group(0) @binding(3)\nvar<storage, read> meshes: Meshes;\n@group(0) @binding(4)\nvar<storage, read> bhv: BHV;\n\n@group(1) @binding(0)\nvar<storage, read_write> commands_count: atomic<u32>;\n@group(1) @binding(1)\nvar<storage, read_write> commands: DrawIndexedCommands;\n@group(1) @binding(2)\nvar<storage, read_write> meshlet_culling_data: array<atomic<u32>>;\n@group(1) @binding(3)\nvar<storage, read_write> processing_data: array<atomic<i32>>;\n\n\nfn extract_scale(m: mat4x4<f32>) -> vec3<f32> \n{\n    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);\n    let sx = length(s[0]);\n    let sy = length(s[1]);\n    let det = determinant(s);\n    var sz = length(s[2]);\n    if (det < 0.) \n    {\n        sz = -sz;\n    }\n    return vec3<f32>(sx, sy, sz);\n}\n\nfn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> \n{\n    if (row == 1u) {\n        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);\n    } else if (row == 2u) {\n        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);\n    } else if (row == 3u) {\n        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);\n    } else {        \n        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);\n    }\n}\n\nfn normalize_plane(plane: vec4<f32>) -> vec4<f32> \n{\n    return (plane / length(plane.xyz));\n}\n\nfn rotate_vector(v: vec3<f32>, orientation: vec4<f32>) -> vec3<f32> \n{\n    return v + 2. * cross(orientation.xyz, cross(orientation.xyz, v) + orientation.w * v);\n}\n\nfn transform_vector(v: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> vec3<f32> \n{\n    return rotate_vector(v, orientation) * scale + position;\n}\n\nfn matrix_from_translation(translation: vec3<f32>) -> mat4x4<f32> {\n    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0),\n                      vec4<f32>(0.0, 1.0, 0.0, 0.0),\n                      vec4<f32>(0.0, 0.0, 1.0, 0.0),\n                      vec4<f32>(translation.x, translation.y, translation.z, 1.0));\n}\n\nfn matrix_from_scale(scale: vec3<f32>) -> mat4x4<f32> {\n    return mat4x4<f32>(vec4<f32>(scale.x, 0.0, 0.0, 0.0),\n                      vec4<f32>(0.0, scale.y, 0.0, 0.0),\n                      vec4<f32>(0.0, 0.0, scale.z, 0.0),\n                      vec4<f32>(0.0, 0.0, 0.0, 1.0));\n}\n\nfn matrix_from_orientation(q: vec4<f32>) -> mat4x4<f32> {\n    let xx = q.x * q.x;\n    let yy = q.y * q.y;\n    let zz = q.z * q.z;\n    let xy = q.x * q.y;\n    let xz = q.x * q.z;\n    let yz = q.y * q.z;\n    let wx = q.w * q.x;\n    let wy = q.w * q.y;\n    let wz = q.w * q.z;\n\n    let m00 = 1.0 - 2.0 * (yy + zz);\n    let m01 = 2.0 * (xy - wz);\n    let m02 = 2.0 * (xz + wy);\n\n    let m10 = 2.0 * (xy + wz);\n    let m11 = 1.0 - 2.0 * (xx + zz);\n    let m12 = 2.0 * (yz - wx);\n\n    let m20 = 2.0 * (xz - wy);\n    let m21 = 2.0 * (yz + wx);\n    let m22 = 1.0 - 2.0 * (xx + yy);\n\n    // Utilizza la funzione mat4x4 per creare la matrice 4x4\n    return mat4x4<f32>(\n        vec4<f32>(m00, m01, m02, 0.0),\n        vec4<f32>(m10, m11, m12, 0.0),\n        vec4<f32>(m20, m21, m22, 0.0),\n        vec4<f32>(0.0, 0.0, 0.0, 1.0)\n    );\n}\n\nfn transform_matrix(position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> mat4x4<f32> {\n    let translation_matrix = matrix_from_translation(position);\n    let rotation_matrix = matrix_from_orientation(orientation);\n    let scale_matrix = matrix_from_scale(scale);    \n    return translation_matrix * rotation_matrix * scale_matrix;\n}\n\nfn matrix_inverse(m: mat4x4<f32>) -> mat4x4<f32> {\n    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];\n    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];\n    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];\n    let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];\n\n    let b00 = a00 * a11 - a01 * a10;\n    let b01 = a00 * a12 - a02 * a10;\n    let b02 = a00 * a13 - a03 * a10;\n    let b03 = a01 * a12 - a02 * a11;\n    let b04 = a01 * a13 - a03 * a11;\n    let b05 = a02 * a13 - a03 * a12;\n    let b06 = a20 * a31 - a21 * a30;\n    let b07 = a20 * a32 - a22 * a30;\n    let b08 = a20 * a33 - a23 * a30;\n    let b09 = a21 * a32 - a22 * a31;\n    let b10 = a21 * a33 - a23 * a31;\n    let b11 = a22 * a33 - a23 * a32;\n\n    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;\n    \n    // Ottimizzazione: Calcola l'inverso del determinante una sola volta\n    let invDet = 1.0 / det;\n\n    return mat4x4<f32>(\n        vec4<f32>((a11 * b11 - a12 * b10 + a13 * b09) * invDet, (a02 * b10 - a01 * b11 - a03 * b09) * invDet, (a31 * b05 - a32 * b04 + a33 * b03) * invDet, (a22 * b04 - a21 * b05 - a23 * b03) * invDet),\n        vec4<f32>((a12 * b08 - a10 * b11 - a13 * b07) * invDet, (a00 * b11 - a02 * b08 + a03 * b07) * invDet, (a32 * b02 - a30 * b05 - a33 * b01) * invDet, (a20 * b05 - a22 * b02 + a23 * b01) * invDet),\n        vec4<f32>((a10 * b10 - a11 * b08 + a13 * b06) * invDet, (a01 * b08 - a00 * b10 - a03 * b06) * invDet, (a30 * b04 - a31 * b02 + a33 * b00) * invDet, (a21 * b02 - a20 * b04 - a23 * b00) * invDet),\n        vec4<f32>((a11 * b07 - a10 * b09 - a12 * b06) * invDet, (a00 * b09 - a01 * b07 + a02 * b06) * invDet, (a31 * b01 - a30 * b03 - a32 * b00) * invDet, (a20 * b03 - a21 * b01 + a22 * b00) * invDet)\n    );\n}\n\nstruct Derivatives {\n    dx: vec3<f32>,\n    dy: vec3<f32>,\n}\n\nfn pixel_to_normalized(image_pixel: vec2<u32>, image_size: vec2<u32>) -> vec2<f32> {\n    return ((vec2<f32>(0.5) + vec2<f32>(image_pixel)) / vec2<f32>(image_size));\n}\nfn clip_to_normalized(clip_coords: vec2<f32>) -> vec2<f32> {\n    return (clip_coords + vec2<f32>(1.)) * vec2<f32>(0.5);\n}\n\nfn pixel_to_clip(image_pixel: vec2<u32>, image_size: vec2<u32>) -> vec2<f32> {\n    var clip_coords = 2. * pixel_to_normalized(image_pixel, image_size) - vec2<f32>(1.);\n    clip_coords.y = -clip_coords.y;\n    return clip_coords;\n}\n\nfn pixel_to_world(image_pixel: vec2<u32>, image_size: vec2<u32>, depth: f32) -> vec3<f32> {\n    let clip_coords = pixel_to_clip(image_pixel, image_size);\n    let world_pos = clip_to_world(clip_coords, depth);\n    return world_pos;\n}\n\nfn clip_to_world(clip_coords: vec2<f32>, depth: f32) -> vec3<f32> {    \n    var world_pos = constant_data.inverse_view_proj * vec4<f32>(clip_coords, depth, 1.);\n    world_pos /= -world_pos.w;\n    return world_pos.xyz;\n}\n\nfn world_to_clip(world_pos: vec3<f32>) -> vec3<f32> {    \n\tlet ndc_pos: vec4<f32> = constant_data.view_proj * vec4<f32>(world_pos, 1.);\n\treturn ndc_pos.xyz / ndc_pos.w;\n}\n\nfn view_pos() -> vec3<f32> {    \n    return clip_to_world(vec2<f32>(0.), 0.);\n}\n\nfn compute_barycentrics_3d(p1: vec3<f32>, p2: vec3<f32>, p3: vec3<f32>, p: vec3<f32>) -> vec3<f32> {\n    let v1 = p - p1;\n    let v2 = p - p2;\n    let v3 = p - p3;\n    \n    let area = length(cross(v1 - v2, v1 - v3)); \n    return vec3<f32>(length(cross(v2, v3)) / area, length(cross(v3, v1)) / area, length(cross(v1, v2)) / area); \n}\n\nfn compute_barycentrics_2d(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, p: vec2<f32>) -> vec3<f32> {\n    let v0 = b - a;\n    let v1 = c - a;\n    let v2 = p - a;\n    \n    let d00 = dot(v0, v0);    \n    let d01 = dot(v0, v1);    \n    let d11 = dot(v1, v1);\n    let d20 = dot(v2, v0);\n    let d21 = dot(v2, v1);\n    \n    let inv_denom = 1. / (d00 * d11 - d01 * d01);    \n    let v = (d11 * d20 - d01 * d21) * inv_denom;    \n    let w = (d00 * d21 - d01 * d20) * inv_denom;    \n    let u = 1. - v - w;\n\n    return vec3 (u,v,w);\n}\n\n// Engel's barycentric coord partial derivs function. Follows equation from [Schied][Dachsbacher]\n// Computes the partial derivatives of point's barycentric coordinates from the projected screen space vertices\nfn compute_partial_derivatives(v0: vec2<f32>, v1: vec2<f32>, v2: vec2<f32>) -> Derivatives\n{\n    let d = 1. / determinant(mat2x2<f32>(v2-v1, v0-v1));\n    \n    return Derivatives(vec3<f32>(v1.y - v2.y, v2.y - v0.y, v0.y - v1.y) * d, vec3<f32>(v2.x - v1.x, v0.x - v2.x, v1.x - v0.x) * d);\n}\n\n// Interpolate 2D attributes using the partial derivatives and generates dx and dy for texture sampling.\nfn interpolate_2d_attribute(a0: vec2<f32>, a1: vec2<f32>, a2: vec2<f32>, deriv: Derivatives, delta: vec2<f32>) -> vec2<f32>\n{\n\tlet attr0 = vec3<f32>(a0.x, a1.x, a2.x);\n\tlet attr1 = vec3<f32>(a0.y, a1.y, a2.y);\n\tlet attribute_x = vec2<f32>(dot(deriv.dx, attr0), dot(deriv.dx, attr1));\n\tlet attribute_y = vec2<f32>(dot(deriv.dy, attr0), dot(deriv.dy, attr1));\n\tlet attribute_s = a0;\n\t\n\treturn (attribute_s + delta.x * attribute_x + delta.y * attribute_y);\n}\n\n// Interpolate vertex attributes at point 'd' using the partial derivatives\nfn interpolate_3d_attribute(a0: vec3<f32>, a1: vec3<f32>, a2: vec3<f32>, deriv: Derivatives, delta: vec2<f32>) -> vec3<f32>\n{\n\tlet attr0 = vec3<f32>(a0.x, a1.x, a2.x);\n\tlet attr1 = vec3<f32>(a0.y, a1.y, a2.y);\n\tlet attr2 = vec3<f32>(a0.z, a1.z, a2.z);\n    let attributes = mat3x3<f32>(a0, a1, a2);\n\tlet attribute_x = attributes * deriv.dx;\n\tlet attribute_y = attributes * deriv.dy;\n\tlet attribute_s = a0;\n\t\n\treturn (attribute_s + delta.x * attribute_x + delta.y * attribute_y);\n}\n\n//ScreenSpace Frustum Culling\nfn is_box_inside_frustum(min: vec3<f32>, max: vec3<f32>, frustum: array<vec4<f32>, 4>) -> bool {\n    var visible: bool = false;    \n    var points: array<vec3<f32>, 8>;\n    points[0] = min;\n    points[1] = max;\n    points[2] = vec3<f32>(min.x, min.y, max.z);\n    points[3] = vec3<f32>(min.x, max.y, max.z);\n    points[4] = vec3<f32>(min.x, max.y, min.z);\n    points[5] = vec3<f32>(max.x, min.y, min.z);\n    points[6] = vec3<f32>(max.x, max.y, min.z);\n    points[7] = vec3<f32>(max.x, min.y, max.z);\n      \n    var f = frustum;\n    for(var i = 0; !visible && i < 4; i = i + 1) {  \n        for(var p = 0; !visible && p < 8; p = p + 1) {        \n            visible = visible || !(dot(f[i].xyz, points[p]) + f[i].w <= 0.);\n        }\n    }   \n    return visible;\n}\n\nvar<workgroup> global_index: atomic<u32>;\nvar<workgroup> global_count: atomic<u32>;\n\n@compute\n@workgroup_size(32, 1, 1)\nfn main(\n    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, \n    @builtin(local_invocation_index) local_invocation_index: u32, \n    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, \n    @builtin(workgroup_id) workgroup_id: vec3<u32>\n) {\n    \n    atomicStore(&global_count, culling_data.lod0_meshlets_count);\n    atomicStore(&commands_count, 0u);\n    \n    if (local_invocation_id.x >= culling_data.lod0_meshlets_count) {\n        return;\n    }\n\n    loop\n    {\n        let index = atomicAdd(&global_index, 1u);        \n        if (index >= atomicLoad(&global_count)) {\n            atomicSub(&global_index, 1u);             \n            break;\n        }\n         \n        let meshlet_id = atomicLoad(&meshlet_culling_data[index]);\n        var desired_lod_level = -1;\n        if(index > culling_data.lod0_meshlets_count) {             \n            desired_lod_level = atomicLoad(&processing_data[meshlet_id]);\n        }\n         \n        let meshlet = meshlets.data[meshlet_id];\n        let mesh_id = meshlet.mesh_index_and_lod_level >> 3u;\n        let mesh = meshes.data[mesh_id];\n        let flags = (mesh.flags_and_vertices_attribute_layout & 0xFFFF0000u) >> 16u;\n        if (flags != culling_data.mesh_flags) {   \n            return;\n        }\n\n        let bb_id = mesh.blas_index + meshlet.bvh_offset;\n        let bb = &bhv.data[bb_id];\n        let bb_max = transform_vector((*bb).max, mesh.position, mesh.orientation, mesh.scale);\n        let bb_min = transform_vector((*bb).min, mesh.position, mesh.orientation, mesh.scale);\n        let min = min(bb_min, bb_max);\n        let max = max(bb_min, bb_max);\n\n        let clip_mvp = constant_data.proj * culling_data.view;\n        let row0 = matrix_row(clip_mvp, 0u);\n        let row1 = matrix_row(clip_mvp, 1u);\n        let row3 = matrix_row(clip_mvp, 3u);\n        var frustum: array<vec4<f32>, 4>;\n        frustum[0] = normalize_plane(row3 + row0);\n        frustum[1] = normalize_plane(row3 - row0);\n        frustum[2] = normalize_plane(row3 + row1);\n        frustum[3] = normalize_plane(row3 - row1);\n        if !is_box_inside_frustum(min, max, frustum) {\n            return;\n        }\n\n        //Evaluate screen occupancy to decide if lod is ok to use for this meshlet or to use childrens\n        if(desired_lod_level < 0) {\n            let ncd_min = clip_mvp * vec4<f32>(min, 1.);\n            let clip_min = ncd_min.xyz / ncd_min.w;\n            let screen_min = clip_to_normalized(clip_min.xy);\n            let ncd_max = clip_mvp * vec4<f32>(max, 1.);\n            let clip_max = ncd_max.xyz / ncd_max.w;\n            let screen_max = clip_to_normalized(clip_max.xy);\n            let screen_diff = (max(screen_max, screen_min) - min(screen_max, screen_min)) * 10.;\n            if clip_min.z > 1. && clip_max.z > 1. {\n                desired_lod_level = 0;\n            }\n            else { \n                let screen_occupancy = max(screen_diff.x, screen_diff.y);     \n                let f_max = f32(MAX_LOD_LEVELS - 1u);   \n                desired_lod_level = i32(clamp(u32(screen_occupancy * f_max), 0u, MAX_LOD_LEVELS - 1u));\n            }\n        }\n        if (constant_data.forced_lod_level >= 0) {\n            desired_lod_level = i32(MAX_LOD_LEVELS - 1 - u32(constant_data.forced_lod_level));\n        }\n\n        let meshlet_lod_level = meshlet.mesh_index_and_lod_level & 7u;\n        let lod_level = u32(desired_lod_level);\n        if(meshlet_lod_level < lod_level) {  \n            let max_lod_level = i32(MAX_LOD_LEVELS);\n            if(meshlet.child_meshlets.x >= 0) {                 \n                let v = atomicMin(&processing_data[meshlet.child_meshlets.x], desired_lod_level);\n                if (v == max_lod_level) {\n                    let child_index = atomicAdd(&global_count, 1u);\n                    atomicStore(&meshlet_culling_data[child_index], u32(meshlet.child_meshlets.x));\n                }\n            }  \n            if(meshlet.child_meshlets.y >= 0) {                 \n                let v = atomicMin(&processing_data[meshlet.child_meshlets.y], desired_lod_level);\n                if (v == max_lod_level) {\n                    let child_index = atomicAdd(&global_count, 1u);                     \n                    atomicStore(&meshlet_culling_data[child_index], u32(meshlet.child_meshlets.y)); \n                }\n            }  \n            if(meshlet.child_meshlets.z >= 0) {                 \n                let v = atomicMin(&processing_data[meshlet.child_meshlets.z], desired_lod_level);\n                if (v == max_lod_level) {\n                    let child_index = atomicAdd(&global_count, 1u);                     \n                    atomicStore(&meshlet_culling_data[child_index], u32(meshlet.child_meshlets.z)); \n                }\n            }  \n            if(meshlet.child_meshlets.w >= 0) {                 \n                let v = atomicMin(&processing_data[meshlet.child_meshlets.w], desired_lod_level);\n                if (v == max_lod_level) {\n                    let child_index = atomicAdd(&global_count, 1u);                     \n                    atomicStore(&meshlet_culling_data[child_index], u32(meshlet.child_meshlets.w)); \n                }  \n            }          \n        } \n        else if(meshlet_lod_level == lod_level)\n        {     \n            let command_index = atomicAdd(&commands_count, 1u);             \n            let command = &commands.data[command_index];\n            (*command).vertex_count = meshlet.indices_count;\n            (*command).instance_count = 1u;\n            (*command).base_index = meshlet.indices_offset;\n            (*command).vertex_offset = i32(mesh.vertices_position_offset);\n            (*command).base_instance = meshlet_id;\n        }\n    }    \n}\n"
}