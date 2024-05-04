const DEFAULT_WIDTH: u32 = 1920u;
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

struct Instance {
    @location(1) mesh_local_bb_min: vec3<f32>,
    @location(2) mesh_index: u32,
    @location(3) mesh_local_bb_max: vec3<f32>,
    @location(4) meshlet_index: u32,
    @location(5) orientation: vec4<f32>,
    @location(6) position: vec3<f32>,
    @location(7) padding1: u32,
    @location(8) scale: vec3<f32>,
    @location(9) padding2: u32,
}

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
    meshlets_offset: u32,
    blas_index: u32,
    indices_offset: u32,
    indices_count: u32,
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

struct Instances {
    data: array<Instance>,
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
    instance_id: u32, 
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
