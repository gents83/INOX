let MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;
let MAX_TEXTURE_COORDS_SET: u32 = 4u;

let TEXTURE_TYPE_BASE_COLOR: u32 = 0u;
let TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;
let TEXTURE_TYPE_NORMAL: u32 = 2u;
let TEXTURE_TYPE_EMISSIVE: u32 = 3u;
let TEXTURE_TYPE_OCCLUSION: u32 = 4u;
let TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;
let TEXTURE_TYPE_DIFFUSE: u32 = 6u;
let TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;
let TEXTURE_TYPE_COUNT: u32 = 8u;

let MATERIAL_ALPHA_BLEND_OPAQUE = 0u;
let MATERIAL_ALPHA_BLEND_MASK = 1u;
let MATERIAL_ALPHA_BLEND_BLEND = 2u;

let MESH_FLAGS_NONE: u32 = 0u;
let MESH_FLAGS_VISIBLE: u32 = 1u;
let MESH_FLAGS_OPAQUE: u32 = 2u; // 1 << 1
let MESH_FLAGS_TRANSPARENT: u32 = 4u;  // 1 << 2
let MESH_FLAGS_WIREFRAME: u32 = 8u; // 1 << 3
let MESH_FLAGS_DEBUG: u32 = 16u; // 1 << 4
let MESH_FLAGS_UI: u32 = 32u; // 1 << 5

let CONSTANT_DATA_FLAGS_NONE: u32 = 0u;
let CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;
let CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 2u;
let CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE: u32 = 4u;
let CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 8u;


struct ConstantData {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    screen_width: f32,
    screen_height: f32,
    flags: u32,
};

struct Vertex {
    @location(0) position_and_color_offset: u32,
    @location(1) normal_offset: i32,
    @location(2) tangent_offset: i32,
    @location(3) mesh_index: u32,
    @location(4) uvs_offset: vec4<i32>,
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

struct Mesh {
    vertex_offset: u32,
    indices_offset: u32,
    material_index: i32,
    mesh_flags: u32,
    position: vec3<f32>,
    meshlets_offset: u32,
    scale: vec3<f32>,
    meshlets_count: u32,
    orientation: vec4<f32>,
};

struct Meshlet {
    @location(5) mesh_index: u32,
    @location(6) aabb_index: u32,
    @location(7) indices_offset: u32,
    @location(8) indices_count: u32,
    @location(9) cone_axis_cutoff: vec4<f32>,
};

struct AABB {
    min: vec3<f32>,
    child_start: i32,
    max: vec3<f32>,
    parent_or_count: i32,
};


struct LightData {
    position: vec3<f32>,
    light_type: u32,
    color: vec4<f32>,
    intensity: f32,
    range: f32,
    inner_cone_angle: f32,
    outer_cone_angle: f32,
};

struct TextureData {
    texture_index: u32,
    layer_index: u32,
    total_width: f32,
    total_height: f32,
    area: vec4<f32>,
};

struct Material {
    textures_indices: array<i32, 8>,//TEXTURE_TYPE_COUNT>,
    textures_coord_set: array<u32, 8>,//TEXTURE_TYPE_COUNT>,
    roughness_factor: f32,
    metallic_factor: f32,
    alpha_cutoff: f32,
    alpha_mode: u32,
    base_color: vec4<f32>,
    emissive_color: vec3<f32>,
    occlusion_strength: f32,
    diffuse_color: vec4<f32>,
    specular_color: vec4<f32>,
};


struct Lights {
    data: array<LightData>,
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

struct Vertices {
    data: array<Vertex>,
};

struct Matrices {
    data: array<mat4x4<f32>>,
};

struct Positions {
    data: array<u32>,
};

struct Colors {
    data: array<u32>,
};

struct Normals {
    data: array<u32>,
};

struct Tangents {
    data: array<vec4<f32>>,
};

struct UVs {
    data: array<u32>,
};

struct AABBs {
    data: array<AABB>,
};

