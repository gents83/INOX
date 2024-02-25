spirv_code  wgsl_code �pconst DEFAULT_WIDTH: u32 = 1920u;
const DEFAULT_HEIGHT: u32 = 1080u;
const SIZE_OF_DATA_BUFFER_ELEMENT: u32 = 4u;
const MAX_LOD_LEVELS: u32 = 8u;
const MAX_NUM_LIGHTS: u32 = 1024u;
const MAX_NUM_TEXTURES: u32 = 65536u;
const MAX_NUM_MATERIALS: u32 = 65536u;

const CONSTANT_DATA_FLAGS_NONE: u32 = 0u;
const CONSTANT_DATA_FLAGS_USE_IBL: u32 = 1u;
const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 1u << 1u;
const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 1u << 2u;
const CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER: u32 = 1u << 3u;
const CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER: u32 = 1u << 4u;
const CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE: u32 = 1u << 5u;
const CONSTANT_DATA_FLAGS_DISPLAY_NORMALS: u32 = 1u << 6u;
const CONSTANT_DATA_FLAGS_DISPLAY_TANGENT: u32 = 1u << 7u;
const CONSTANT_DATA_FLAGS_DISPLAY_BITANGENT: u32 = 1u << 8u;
const CONSTANT_DATA_FLAGS_DISPLAY_BASE_COLOR: u32 = 1u << 9u;
const CONSTANT_DATA_FLAGS_DISPLAY_METALLIC: u32 = 1u << 10u;
const CONSTANT_DATA_FLAGS_DISPLAY_ROUGHNESS: u32 = 1u << 11u;
const CONSTANT_DATA_FLAGS_DISPLAY_UV_0: u32 = 1u << 12u;
const CONSTANT_DATA_FLAGS_DISPLAY_UV_1: u32 = 1u << 13u;
const CONSTANT_DATA_FLAGS_DISPLAY_UV_2: u32 = 1u << 14u;
const CONSTANT_DATA_FLAGS_DISPLAY_UV_3: u32 = 1u << 15u;

const MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;
const MAX_TEXTURE_COORDS_SET: u32 = 4u;

const TEXTURE_TYPE_BASE_COLOR: u32 = 0u;
const TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;
const TEXTURE_TYPE_NORMAL: u32 = 2u;
const TEXTURE_TYPE_EMISSIVE: u32 = 3u;
const TEXTURE_TYPE_OCCLUSION: u32 = 4u;
const TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;
const TEXTURE_TYPE_DIFFUSE: u32 = 6u;
const TEXTURE_TYPE_SPECULAR: u32 = 7u;
const TEXTURE_TYPE_SPECULAR_COLOR: u32 = 8u;
const TEXTURE_TYPE_TRANSMISSION: u32 = 9u;
const TEXTURE_TYPE_THICKNESS: u32 = 10u;
const TEXTURE_TYPE_EMPTY_FOR_PADDING_3: u32 = 11u;
const TEXTURE_TYPE_EMPTY_FOR_PADDING_4: u32 = 12u;
const TEXTURE_TYPE_EMPTY_FOR_PADDING_5: u32 = 13u;
const TEXTURE_TYPE_EMPTY_FOR_PADDING_6: u32 = 14u;
const TEXTURE_TYPE_EMPTY_FOR_PADDING_7: u32 = 15u;
const TEXTURE_TYPE_COUNT: u32 = 16u;

const MATERIAL_ALPHA_BLEND_OPAQUE = 0u;
const MATERIAL_ALPHA_BLEND_MASK = 1u;
const MATERIAL_ALPHA_BLEND_BLEND = 2u;

const MESH_FLAGS_NONE: u32 = 0u;
const MESH_FLAGS_VISIBLE: u32 = 1u;
const MESH_FLAGS_OPAQUE: u32 = 1u << 1u;
const MESH_FLAGS_TRANSPARENT: u32 = 1u << 2u;
const MESH_FLAGS_WIREFRAME: u32 = 1u << 3u;
const MESH_FLAGS_DEBUG: u32 = 1u << 4u;
const MESH_FLAGS_UI: u32 = 1u << 5u;


const MATH_PI: f32 = 3.14159265359;
const MATH_EPSILON = 0.0000001;
const MAX_FLOAT: f32 = 3.402823466e+38;
const MAX_TRACING_DISTANCE: f32 = 500.;
const HIT_EPSILON: f32 = 0.0001;
const INVALID_NODE: i32 = -1;

const VERTEX_ATTRIBUTE_HAS_POSITION: u32 = 0u;
const VERTEX_ATTRIBUTE_HAS_COLOR: u32 = 1u;
const VERTEX_ATTRIBUTE_HAS_NORMAL: u32 = 1u << 1u;
const VERTEX_ATTRIBUTE_HAS_TANGENT: u32 = 1u << 2u;
const VERTEX_ATTRIBUTE_HAS_UV1: u32 = 1u << 3u;
const VERTEX_ATTRIBUTE_HAS_UV2: u32 = 1u << 4u;
const VERTEX_ATTRIBUTE_HAS_UV3: u32 = 1u << 5u;
const VERTEX_ATTRIBUTE_HAS_UV4: u32 = 1u << 6u;

const MATERIAL_FLAGS_NONE: u32 = 0u;
const MATERIAL_FLAGS_UNLIT: u32 = 1u;
const MATERIAL_FLAGS_IRIDESCENCE: u32 = 1u << 1u;
const MATERIAL_FLAGS_ANISOTROPY: u32 = 1u << 2u;
const MATERIAL_FLAGS_CLEARCOAT: u32 = 1u << 3u;
const MATERIAL_FLAGS_SHEEN: u32 = 1u << 4u;
const MATERIAL_FLAGS_TRANSMISSION: u32 = 1u << 5u;
const MATERIAL_FLAGS_VOLUME: u32 = 1u << 6u;
const MATERIAL_FLAGS_EMISSIVE_STRENGTH: u32 = 1u << 7u;
const MATERIAL_FLAGS_METALLICROUGHNESS: u32 = 1u << 8u;
const MATERIAL_FLAGS_SPECULAR: u32 = 1u << 9u;
const MATERIAL_FLAGS_SPECULARGLOSSINESS: u32 = 1u << 10u;
const MATERIAL_FLAGS_IOR: u32 = 1u << 11u;
const MATERIAL_FLAGS_ALPHAMODE_OPAQUE: u32 = 1u << 12u;
const MATERIAL_FLAGS_ALPHAMODE_MASK: u32 = 1u << 13u;
const MATERIAL_FLAGS_ALPHAMODE_BLEND: u32 = 1u << 14u;

const LIGHT_TYPE_INVALID: u32 = 0u;
const LIGHT_TYPE_DIRECTIONAL: u32 = 1u;
const LIGHT_TYPE_POINT: u32 = 1u << 1u;
const LIGHT_TYPE_SPOT: u32 = 1u << 2u;

struct ConstantData {
    view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    screen_width: f32,
    screen_height: f32,
    frame_index: u32,
    flags: u32,
    debug_uv_coords: vec2<f32>,
    tlas_starting_index: u32,
    indirect_light_num_bounces: u32,
    lut_pbr_charlie_texture_index: u32,
    lut_pbr_ggx_texture_index: u32,
    environment_map_texture_index: u32,
    num_lights: u32,
    forced_lod_level: i32,
    camera_near: f32,
    camera_far: f32,
    _empty3: u32,
};

struct RuntimeVertexData {
    @location(0) world_pos: vec3<f32>,
    @location(1) @interpolate(flat) mesh_index: u32,
};

struct DrawCommand {
    vertex_count: u32,
    instance_count: u32,
    base_vertex: u32,
    base_instance: u32,
};

struct DrawIndexedCommand {
    vertex_count: u32,
    instance_count: u32,
    base_index: u32,
    vertex_offset: i32,
    base_instance: u32,
};

struct DispatchCommandSize {
    x: atomic<u32>,
    y: atomic<u32>,
    z: atomic<u32>,
};

struct Mesh {
    vertices_position_offset: u32,
    vertices_attribute_offset: u32,
    flags_and_vertices_attribute_layout: u32,
    material_index: i32,
    orientation: vec4<f32>,
    position: vec3<f32>,
    meshlets_offset: u32,
    scale: vec3<f32>,
    blas_index: u32,
    lods_meshlets_offset: array<u32, MAX_LOD_LEVELS>,
};

struct Meshlet {
    @location(5) mesh_index_and_lod_level: u32, // 29 mesh + 3 lod bits
    @location(6) indices_offset: u32,
    @location(7) indices_count: u32,
    @location(8) bvh_offset: u32,
    @location(9) child_meshlets: vec4<i32>,
};

struct BHVNode {
    min: vec3<f32>,
    miss: i32,
    max: vec3<f32>,
    reference: i32, //-1 or mesh_index or meshlet_index or triangle_index
};


struct LightData {
    position: vec3<f32>,
    light_type: u32,
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    range: f32,
    inner_cone_angle: f32,
    outer_cone_angle: f32,
    _padding1: f32,
    _padding2: f32,
};

struct TextureData {
    texture_and_layer_index: i32,
    min: u32,
    max: u32,
    size: u32,
};

struct Material {
    roughness_factor: f32,
    metallic_factor: f32,
    ior: f32,
    transmission_factor: f32,
    base_color: vec4<f32>,
    emissive_color: vec3<f32>,
    emissive_strength: f32,
    diffuse_color: vec4<f32>,
    specular_color: vec4<f32>,
    specular_factors: vec4<f32>,
    attenuation_color_and_distance: vec4<f32>,
    thickness_factor: f32,
    normal_scale_and_alpha_cutoff: u32,
    occlusion_strength: f32,
    flags: u32,
    textures_index_and_coord_set: array<u32, TEXTURE_TYPE_COUNT>,
};


struct Lights {
    data: array<LightData, MAX_NUM_LIGHTS>,
};

struct Textures {
    data: array<TextureData>,
};

struct Materials {
    data: array<Material>,
};

struct DrawCommands {
    data: array<DrawCommand>,
};

struct DrawIndexedCommands {
    data: array<DrawIndexedCommand>,
};

struct Meshes {
    data: array<Mesh>,
};

struct Meshlets {
    data: array<Meshlet>,
};

struct Indices {
    data: array<u32>,
};

struct RuntimeVertices {
    data: array<RuntimeVertexData>,
};

struct VerticesPositions {
    data: array<u32>,
};

struct VerticesAttributes {
    data: array<u32>,
};

struct BHV {
    data: array<BHVNode>,
};


struct Ray {
    origin: vec3<f32>,
    t_min: f32,
    direction: vec3<f32>,
    t_max: f32,
};

struct PixelData {
    world_pos: vec3<f32>,
    material_id: u32,
    color: vec4<f32>,
    normal: vec3<f32>,
    mesh_id: u32, 
    tangent: vec4<f32>,
    uv_set: array<vec4<f32>, 4>,
};

struct TBN {
    normal: vec3<f32>,
    tangent: vec3<f32>,
    binormal: vec3<f32>,
};

struct MaterialInfo {
    base_color: vec4<f32>,

    f0: vec3<f32>,
    ior: f32,

    c_diff: vec3<f32>,
    perceptual_roughness: f32,

    metallic: f32,
    specular_weight_and_anisotropy_strength: u32,
    transmission_factor: f32,
    thickness_factor: f32,

    attenuation_color_and_distance: vec4<f32>,
    sheen_color_and_roughness_factor: vec4<f32>,

    clear_coat_f0: vec3<f32>,
    clear_coat_factor: f32,

    clear_coat_f90: vec3<f32>,
    clear_coat_roughness_factor: f32,

    clear_coat_normal: vec3<f32>,
    iridescence_factor: f32,

    anisotropicT: vec3<f32>,
    iridescence_ior: f32,

    anisotropicB: vec3<f32>,
    iridescence_thickness: f32,

    alpha_roughness: f32,
    f90: vec3<f32>,
    
    f_color: vec4<f32>,
    f_emissive: vec3<f32>,
    f_diffuse: vec3<f32>,
    f_diffuse_ibl: vec3<f32>,
    f_specular: vec3<f32>,
};
fn quantize_unorm(v: f32, n: u32) -> u32 {
    let scale = f32((1u << n) - 1u);
    return u32(0.5 + (v * scale));
}
fn quantize_snorm(v: f32, n: u32) -> u32 {
    let c = (1u << (n - 1u)) - 1u;
    let scale = f32(c);
    if v < 0. {
        return (u32(-v * scale) & c) | (1u << (n - 1u));
    } else {
        return u32(v * scale) & c;
    }
}

fn decode_unorm(i: u32, n: u32) -> f32 {    
    let scale = f32((1u << n) - 1u);
    if (i == 0u) {
        return 0.;
    } else if (i == u32(scale)) {
        return 1.;
    } else {
        return (f32(i) - 0.5) / scale;
    }
}

fn decode_snorm(i: u32, n: u32) -> f32 {
    let s = i >> (n - 1u);
    let c = (1u << (n - 1u)) - 1u;
    let scale = f32(c);
    if s > 0u {
        let r = f32(i & c) / scale;
        return -r;
    } else {
        return f32(i & c) / scale;
    }
}

fn pack_3_f32_to_unorm(value: vec3<f32>) -> u32 {
    let x = quantize_unorm(value.x, 10u) << 20u;
    let y = quantize_unorm(value.y, 10u) << 10u;
    let z = quantize_unorm(value.z, 10u);
    return (x | y | z);
}
fn unpack_unorm_to_3_f32(v: u32) -> vec3<f32> {
    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);
    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);
    let vz = decode_unorm(v & 0x000003FFu, 10u);
    return vec3<f32>(vx, vy, vz);
}

fn pack_3_f32_to_snorm(value: vec3<f32>) -> u32 {
    let x = quantize_snorm(value.x, 10u) << 20u;
    let y = quantize_snorm(value.y, 10u) << 10u;
    let z = quantize_snorm(value.z, 10u);
    return (x | y | z);
}
fn unpack_snorm_to_3_f32(v: u32) -> vec3<f32> {
    let vx = decode_snorm((v >> 20u) & 0x000003FFu, 10u);
    let vy = decode_snorm((v >> 10u) & 0x000003FFu, 10u);
    let vz = decode_snorm(v & 0x000003FFu, 10u);
    return vec3<f32>(vx, vy, vz);
}

fn unpack_normal(f: f32) -> vec3<f32> {
	var f_var = f;
	var flipZ: f32 = sign(f_var);
	f_var = abs(f_var);
	let atanXY: f32 = floor(f_var) / 67.5501 * (3.1415927 * 2.) - 3.1415927;
	var n: vec3<f32> = vec3<f32>(sin(atanXY), cos(atanXY), 0.);
	n.z = fract(f_var) * 1869.2296 / 427.67993;
	n = normalize(n);
	n.z = n.z * (flipZ);
	return n;
} 

fn pack_normal(n: vec3<f32>) -> f32 {
	var n_var = n;
	let flipZ: f32 = sign(n_var.z);
	n_var.z = abs(n_var.z);
	n_var = n_var / (23.065746);
	let xy: f32 = floor((atan2(n_var.x, n_var.y) + 3.1415927) / (3.1415927 * 2.) * 67.5501);
	var z: f32 = floor(n_var.z * 427.67993) / 1869.2296;
	z = z * (1. / max(0.01, length(vec2<f32>(n_var.x, n_var.y))));
	return (xy + z) * flipZ;
} 


fn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {
    let r = quantize_unorm(value.x, 8u) << 24u;
    let g = quantize_unorm(value.y, 8u) << 16u;
    let b = quantize_unorm(value.z, 8u) << 8u;
    let a = quantize_unorm(value.w, 8u);
    return (r | g | b | a);
}
fn unpack_snorm_to_4_f32(v: u32) -> vec4<f32> {
    let r = decode_snorm((v >> 24u) & 255u, 8u);
    let g = decode_snorm((v >> 16u) & 255u, 8u);
    let b = decode_snorm((v >> 8u) & 255u, 8u);
    let a = decode_snorm(v & 255u, 8u);
    return vec4<f32>(r,g,b,a);
}
fn unpack_unorm_to_4_f32(v: u32) -> vec4<f32> {
    let r = decode_unorm((v >> 24u) & 255u, 8u);
    let g = decode_unorm((v >> 16u) & 255u, 8u);
    let b = decode_unorm((v >> 8u) & 255u, 8u);
    let a = decode_unorm(v & 255u, 8u);
    return vec4<f32>(r,g,b,a);
}

fn sign_not_zero(v: vec2<f32>) -> vec2<f32> {
	return vec2<f32>(select(-1., 1., v.x >= 0.), select(-1., 1., v.y >= 0.));
} 

fn octahedral_mapping(v: vec3<f32>) -> vec2<f32> {
	let l1norm: f32 = abs(v.x) + abs(v.y) + abs(v.z);
	var result: vec2<f32> = v.xy * (1. / l1norm);
	if (v.z < 0.) {
		result = (1. - abs(result.yx)) * sign_not_zero(result.xy);
	}
	return result;
} 

fn octahedral_unmapping(o: vec2<f32>) -> vec3<f32> {
	var v: vec3<f32> = vec3<f32>(o.x, o.y, 1. - abs(o.x) - abs(o.y));
	if (v.z < 0.) {
		var vxy = v.xy;
        vxy = (1. - abs(v.yx)) * sign_not_zero(v.xy);
        v.x = vxy.x;
        v.y = vxy.y;
	}
	return normalize(v);
} 

fn f32tof16(v: f32) -> u32 {
    return pack2x16float(vec2<f32>(v, 0.));
}

fn f16tof32(v: u32) -> f32 {
    return unpack2x16float(v & 0x0000FFFFu).x;
}

fn pack_into_R11G11B10F(rgb: vec3<f32>) -> u32 {
	let r = (f32tof16(rgb.r) << 17u) & 0xFFE00000u;
	let g = (f32tof16(rgb.g) << 6u) & 0x001FFC00u;
	let b = (f32tof16(rgb.b) >> 5u) & 0x000003FFu;
	return r | g | b;
} 

fn unpack_from_R11G11B10F(rgb: u32) -> vec3<f32> {
	let r = f16tof32((rgb >> 17u) & 0x7FF0u);
	let g = f16tof32((rgb >> 6u) & 0x7FF0u);
	let b = f16tof32((rgb << 5u) & 0x7FE0u);
	return vec3<f32>(r, g, b);
} 


fn iq_hash(v: vec2<f32>) -> f32 {
    return fract(sin(dot(v, vec2(11.9898, 78.233))) * 43758.5453);
}
fn blue_noise(in: vec2<f32>) -> f32 {
    var v =  iq_hash( in + vec2<f32>(-1., 0.) )
             + iq_hash( in + vec2<f32>( 1., 0.) )
             + iq_hash( in + vec2<f32>( 0., 1.) )
             + iq_hash( in + vec2<f32>( 0.,-1.) ); 
    v /= 4.;
    return (iq_hash(in) - v + .5);
}

// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
fn hash( x: u32 ) -> u32 {
    var v = x;
    v += ( v << 10u );
    v ^= ( v >>  6u );
    v += ( v <<  3u );
    v ^= ( v >> 11u );
    v += ( v << 15u );
    return v;
}

fn hash1(seed: f32) -> f32 {
    var p = fract(seed * .1031);
    p *= p + 33.33;
    p *= p + p;
    return fract(p);
}

fn hash2(seed: ptr<function, f32>) -> vec2<f32> {
    let a = (*seed) + 0.1;
    let b = a + 0.1;
    (*seed) = b;
    return fract(sin(vec2(a,b))*vec2(43758.5453123,22578.1459123));
}

fn hash3(seed: ptr<function, f32>) -> vec3<f32> {
    let a = (*seed) + 0.1;
    let b = a + 0.1;
    let c = b + 0.1;
    (*seed) = c;
    return fract(sin(vec3(a,b,c))*vec3(43758.5453123,22578.1459123,19642.3490423));
}

// This is PCG2d
fn get_random_numbers(seed: ptr<function, vec2<u32>>) -> vec2<f32> {
    var v = (*seed) * 1664525u + 1013904223u;
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    *seed = v;
    return vec2<f32>(v) * 2.32830643654e-10;
}

fn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) 
{
    let c = *ptr_a;
    *ptr_a = *ptr_b;
    *ptr_b = c;
}

fn mod_f32(v: f32, m: f32) -> f32
{
    return v - (m * floor(v/m));
}

fn clamped_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return clamp(dot(a,b), 0., 1.);
}

fn has_vertex_attribute(vertex_attribute_layout: u32, attribute_to_check: u32) -> bool {
    return bool(vertex_attribute_layout & attribute_to_check);
}
fn vertex_attribute_offset(vertex_attribute_layout: u32, attribute_to_check: u32) -> i32 
{
    if(has_vertex_attribute(vertex_attribute_layout, attribute_to_check)) {
        let mask = (vertex_attribute_layout & 0x0000FFFFu) & (~attribute_to_check & (attribute_to_check - 1u));
        return i32(countOneBits(mask));
    }
    return -1;
}
fn vertex_layout_stride(vertex_attribute_layout: u32) -> u32 
{
    return countOneBits((vertex_attribute_layout & 0x0000FFFFu));
}

struct CullingData {
    view: mat4x4<f32>,
    mesh_flags: u32,
    lod0_meshlets_count: u32,
    _padding1: u32,
    _padding2: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> culling_data: CullingData;
@group(0) @binding(2)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(3)
var<storage, read> meshes: Meshes;
@group(0) @binding(4)
var<storage, read> bhv: BHV;
@group(0) @binding(5)
var<storage, read_write> meshlets_lod_level: array<atomic<u32>>;


fn extract_scale(m: mat4x4<f32>) -> vec3<f32> 
{
    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
    let sx = length(s[0]);
    let sy = length(s[1]);
    let det = determinant(s);
    var sz = length(s[2]);
    if (det < 0.) 
    {
        sz = -sz;
    }
    return vec3<f32>(sx, sy, sz);
}

fn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> 
{
    if (row == 1u) {
        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);
    } else if (row == 2u) {
        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);
    } else if (row == 3u) {
        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);
    } else {        
        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);
    }
}

fn normalize_plane(plane: vec4<f32>) -> vec4<f32> 
{
    return (plane / length(plane.xyz));
}

fn rotate_vector(v: vec3<f32>, orientation: vec4<f32>) -> vec3<f32> 
{
    return v + 2. * cross(orientation.xyz, cross(orientation.xyz, v) + orientation.w * v);
}

fn transform_vector(v: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> vec3<f32> 
{
    return rotate_vector(v, orientation) * scale + position;
}

fn matrix_from_translation(translation: vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0),
                      vec4<f32>(0.0, 1.0, 0.0, 0.0),
                      vec4<f32>(0.0, 0.0, 1.0, 0.0),
                      vec4<f32>(translation.x, translation.y, translation.z, 1.0));
}

fn matrix_from_scale(scale: vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(scale.x, 0.0, 0.0, 0.0),
                      vec4<f32>(0.0, scale.y, 0.0, 0.0),
                      vec4<f32>(0.0, 0.0, scale.z, 0.0),
                      vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

fn matrix_from_orientation(q: vec4<f32>) -> mat4x4<f32> {
    let xx = q.x * q.x;
    let yy = q.y * q.y;
    let zz = q.z * q.z;
    let xy = q.x * q.y;
    let xz = q.x * q.z;
    let yz = q.y * q.z;
    let wx = q.w * q.x;
    let wy = q.w * q.y;
    let wz = q.w * q.z;

    let m00 = 1.0 - 2.0 * (yy + zz);
    let m01 = 2.0 * (xy - wz);
    let m02 = 2.0 * (xz + wy);

    let m10 = 2.0 * (xy + wz);
    let m11 = 1.0 - 2.0 * (xx + zz);
    let m12 = 2.0 * (yz - wx);

    let m20 = 2.0 * (xz - wy);
    let m21 = 2.0 * (yz + wx);
    let m22 = 1.0 - 2.0 * (xx + yy);

    // Utilizza la funzione mat4x4 per creare la matrice 4x4
    return mat4x4<f32>(
        vec4<f32>(m00, m01, m02, 0.0),
        vec4<f32>(m10, m11, m12, 0.0),
        vec4<f32>(m20, m21, m22, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

fn transform_matrix(position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> mat4x4<f32> {
    let translation_matrix = matrix_from_translation(position);
    let rotation_matrix = matrix_from_orientation(orientation);
    let scale_matrix = matrix_from_scale(scale);    
    return translation_matrix * rotation_matrix * scale_matrix;
}

fn matrix_inverse(m: mat4x4<f32>) -> mat4x4<f32> {
    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];
    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];
    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];
    let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];

    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;

    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    
    // Ottimizzazione: Calcola l'inverso del determinante una sola volta
    let invDet = 1.0 / det;

    return mat4x4<f32>(
        vec4<f32>((a11 * b11 - a12 * b10 + a13 * b09) * invDet, (a02 * b10 - a01 * b11 - a03 * b09) * invDet, (a31 * b05 - a32 * b04 + a33 * b03) * invDet, (a22 * b04 - a21 * b05 - a23 * b03) * invDet),
        vec4<f32>((a12 * b08 - a10 * b11 - a13 * b07) * invDet, (a00 * b11 - a02 * b08 + a03 * b07) * invDet, (a32 * b02 - a30 * b05 - a33 * b01) * invDet, (a20 * b05 - a22 * b02 + a23 * b01) * invDet),
        vec4<f32>((a10 * b10 - a11 * b08 + a13 * b06) * invDet, (a01 * b08 - a00 * b10 - a03 * b06) * invDet, (a30 * b04 - a31 * b02 + a33 * b00) * invDet, (a21 * b02 - a20 * b04 - a23 * b00) * invDet),
        vec4<f32>((a11 * b07 - a10 * b09 - a12 * b06) * invDet, (a00 * b09 - a01 * b07 + a02 * b06) * invDet, (a31 * b01 - a30 * b03 - a32 * b00) * invDet, (a20 * b03 - a21 * b01 + a22 * b00) * invDet)
    );
}

struct Derivatives {
    dx: vec3<f32>,
    dy: vec3<f32>,
}

fn pixel_to_normalized(image_pixel: vec2<u32>, image_size: vec2<u32>) -> vec2<f32> {
    return ((vec2<f32>(0.5) + vec2<f32>(image_pixel)) / vec2<f32>(image_size));
}
fn clip_to_normalized(clip_coords: vec2<f32>) -> vec2<f32> {
    return (clip_coords + vec2<f32>(1.)) * vec2<f32>(0.5);
}

fn pixel_to_clip(image_pixel: vec2<u32>, image_size: vec2<u32>) -> vec2<f32> {
    var clip_coords = 2. * pixel_to_normalized(image_pixel, image_size) - vec2<f32>(1.);
    clip_coords.y *= -1.;
    return clip_coords;
}

fn pixel_to_world(image_pixel: vec2<u32>, image_size: vec2<u32>, depth: f32) -> vec3<f32> {
    let clip_coords = pixel_to_clip(image_pixel, image_size);
    let world_pos = clip_to_world(clip_coords, depth);
    return world_pos;
}

fn clip_to_world(clip_coords: vec2<f32>, depth: f32) -> vec3<f32> {    
    var world_pos = constant_data.inverse_view_proj * vec4<f32>(clip_coords, depth, 1.);
    world_pos /= world_pos.w;
    return world_pos.xyz;
}

fn world_to_clip(world_pos: vec3<f32>) -> vec3<f32> {    
	let ndc_pos: vec4<f32> = constant_data.view_proj * vec4<f32>(world_pos, 1.);
	return ndc_pos.xyz / ndc_pos.w;
}

fn view_pos() -> vec3<f32> {    
    return clip_to_world(vec2<f32>(0.), 0.);
}

fn compute_barycentrics_3d(p1: vec3<f32>, p2: vec3<f32>, p3: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    let v1 = p - p1;
    let v2 = p - p2;
    let v3 = p - p3;
    
    let area = length(cross(v1 - v2, v1 - v3)); 
    return vec3<f32>(length(cross(v2, v3)) / area, length(cross(v3, v1)) / area, length(cross(v1, v2)) / area); 
}

fn compute_barycentrics_2d(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, p: vec2<f32>) -> vec3<f32> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    
    let d00 = dot(v0, v0);    
    let d01 = dot(v0, v1);    
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    
    let inv_denom = 1. / (d00 * d11 - d01 * d01);    
    let v = (d11 * d20 - d01 * d21) * inv_denom;    
    let w = (d00 * d21 - d01 * d20) * inv_denom;    
    let u = 1. - v - w;

    return vec3 (u,v,w);
}

// Engel's barycentric coord partial derivs function. Follows equation from [Schied][Dachsbacher]
// Computes the partial derivatives of point's barycentric coordinates from the projected screen space vertices
fn compute_partial_derivatives(v0: vec2<f32>, v1: vec2<f32>, v2: vec2<f32>) -> Derivatives
{
    let d = 1. / determinant(mat2x2<f32>(v2-v1, v0-v1));
    
    return Derivatives(vec3<f32>(v1.y - v2.y, v2.y - v0.y, v0.y - v1.y) * d, vec3<f32>(v2.x - v1.x, v0.x - v2.x, v1.x - v0.x) * d);
}

// Interpolate 2D attributes using the partial derivatives and generates dx and dy for texture sampling.
fn interpolate_2d_attribute(a0: vec2<f32>, a1: vec2<f32>, a2: vec2<f32>, deriv: Derivatives, delta: vec2<f32>) -> vec2<f32>
{
	let attr0 = vec3<f32>(a0.x, a1.x, a2.x);
	let attr1 = vec3<f32>(a0.y, a1.y, a2.y);
	let attribute_x = vec2<f32>(dot(deriv.dx, attr0), dot(deriv.dx, attr1));
	let attribute_y = vec2<f32>(dot(deriv.dy, attr0), dot(deriv.dy, attr1));
	let attribute_s = a0;
	
	return (attribute_s + delta.x * attribute_x + delta.y * attribute_y);
}

// Interpolate vertex attributes at point 'd' using the partial derivatives
fn interpolate_3d_attribute(a0: vec3<f32>, a1: vec3<f32>, a2: vec3<f32>, deriv: Derivatives, delta: vec2<f32>) -> vec3<f32>
{
	let attr0 = vec3<f32>(a0.x, a1.x, a2.x);
	let attr1 = vec3<f32>(a0.y, a1.y, a2.y);
	let attr2 = vec3<f32>(a0.z, a1.z, a2.z);
    let attributes = mat3x3<f32>(a0, a1, a2);
	let attribute_x = attributes * deriv.dx;
	let attribute_y = attributes * deriv.dy;
	let attribute_s = a0;
	
	return (attribute_s + delta.x * attribute_x + delta.y * attribute_y);
}

//ScreenSpace Frustum Culling
fn is_box_inside_frustum(min: vec3<f32>, max: vec3<f32>, frustum: array<vec4<f32>, 4>) -> bool {
    var visible: bool = false;    
    var points: array<vec3<f32>, 8>;
    points[0] = min;
    points[1] = max;
    points[2] = vec3<f32>(min.x, min.y, max.z);
    points[3] = vec3<f32>(min.x, max.y, max.z);
    points[4] = vec3<f32>(min.x, max.y, min.z);
    points[5] = vec3<f32>(max.x, min.y, min.z);
    points[6] = vec3<f32>(max.x, max.y, min.z);
    points[7] = vec3<f32>(max.x, min.y, max.z);
      
    var f = frustum;
    for(var i = 0; !visible && i < 4; i = i + 1) {  
        for(var p = 0; !visible && p < 8; p = p + 1) {        
            visible = visible || !(dot(f[i].xyz, points[p]) + f[i].w <= 0.);
        }
    }   
    return visible;
}

@compute
@workgroup_size(32, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let meshlet_id = global_invocation_id.x;
    if (meshlet_id >= arrayLength(&meshlets.data)) {
        return;
    }

    let meshlet = meshlets.data[meshlet_id];
    let mesh_id = meshlet.mesh_index_and_lod_level >> 3u;
    let mesh = meshes.data[mesh_id];
    let flags = (mesh.flags_and_vertices_attribute_layout & 0xFFFF0000u) >> 16u;
    if (flags != culling_data.mesh_flags) {   
        atomicStore(&meshlets_lod_level[meshlet_id], MAX_LOD_LEVELS);
        return;
    }

    let bb_id = mesh.blas_index + meshlet.bvh_offset;
    let bb = &bhv.data[bb_id];
    let bb_max = transform_vector((*bb).max, mesh.position, mesh.orientation, mesh.scale);
    let bb_min = transform_vector((*bb).min, mesh.position, mesh.orientation, mesh.scale);
    let min = min(bb_min, bb_max);
    let max = max(bb_min, bb_max);

    let clip_mvp = constant_data.proj * culling_data.view;
    let row0 = matrix_row(clip_mvp, 0u);
    let row1 = matrix_row(clip_mvp, 1u);
    let row3 = matrix_row(clip_mvp, 3u);
    var frustum: array<vec4<f32>, 4>;
    frustum[0] = normalize_plane(row3 + row0);
    frustum[1] = normalize_plane(row3 - row0);
    frustum[2] = normalize_plane(row3 + row1);
    frustum[3] = normalize_plane(row3 - row1);
    if !is_box_inside_frustum(min, max, frustum) {
        atomicStore(&meshlets_lod_level[meshlet_id], MAX_LOD_LEVELS);
        return;
    }

    //Evaluate screen occupancy to decide if lod is ok to use for this meshlet or to use childrens
    var screen_lod_level = 0u;   
    let f_max = f32(MAX_LOD_LEVELS);   

    let ncd_min = clip_mvp * vec4<f32>(min, 1.);
    let clip_min = ncd_min.xyz / ncd_min.w;
    let screen_min = clip_to_normalized(clip_min.xy);
    let ncd_max = clip_mvp * vec4<f32>(max, 1.);
    let clip_max = ncd_max.xyz / ncd_max.w;
    let screen_max = clip_to_normalized(clip_max.xy);
    let screen_diff = max(screen_max, screen_min) - min(screen_max, screen_min);
    let screen_occupancy = clamp(max(screen_diff.x, screen_diff.y), 0., 1.);  
    screen_lod_level =  clamp(u32(screen_occupancy * f_max), 0u, MAX_LOD_LEVELS - 1u);

    let center = min + (max-min) * 0.5;
    let distance = length(view_pos() - center);
    let distance_lod_level = MAX_LOD_LEVELS - 1u - clamp(u32(((distance * distance) / (constant_data.camera_far - constant_data.camera_near)) * f_max), 0u, MAX_LOD_LEVELS - 1u);

    var desired_lod_level = max(distance_lod_level, screen_lod_level);

    if (constant_data.forced_lod_level >= 0) {
        desired_lod_level = MAX_LOD_LEVELS - 1u - u32(constant_data.forced_lod_level);
    }
    
    atomicMax(&meshlets_lod_level[meshlet_id], desired_lod_level);
    
    if(meshlet.child_meshlets.x >= 0) {
        atomicMax(&meshlets_lod_level[meshlet.child_meshlets.x], desired_lod_level);
    }
    if(meshlet.child_meshlets.y >= 0) {
        atomicMax(&meshlets_lod_level[meshlet.child_meshlets.y], desired_lod_level);
    }
    if(meshlet.child_meshlets.z >= 0) {
        atomicMax(&meshlets_lod_level[meshlet.child_meshlets.z], desired_lod_level);
    }
    if(meshlet.child_meshlets.w >= 0) {
        atomicMax(&meshlets_lod_level[meshlet.child_meshlets.w], desired_lod_level);
    }
}
  �p�p   �p�p(%