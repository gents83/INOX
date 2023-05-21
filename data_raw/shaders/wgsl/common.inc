const MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;
const MAX_TEXTURE_COORDS_SET: u32 = 4u;

const TEXTURE_TYPE_BASE_COLOR: u32 = 0u;
const TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;
const TEXTURE_TYPE_NORMAL: u32 = 2u;
const TEXTURE_TYPE_EMISSIVE: u32 = 3u;
const TEXTURE_TYPE_OCCLUSION: u32 = 4u;
const TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;
const TEXTURE_TYPE_DIFFUSE: u32 = 6u;
const TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;
const TEXTURE_TYPE_COUNT: u32 = 8u;

const MATERIAL_ALPHA_BLEND_OPAQUE = 0u;
const MATERIAL_ALPHA_BLEND_MASK = 1u;
const MATERIAL_ALPHA_BLEND_BLEND = 2u;

const MESH_FLAGS_NONE: u32 = 0u;
const MESH_FLAGS_VISIBLE: u32 = 1u;
const MESH_FLAGS_OPAQUE: u32 = 2u; // 1 << 1
const MESH_FLAGS_TRANSPARENT: u32 = 4u;  // 1 << 2
const MESH_FLAGS_WIREFRAME: u32 = 8u; // 1 << 3
const MESH_FLAGS_DEBUG: u32 = 16u; // 1 << 4
const MESH_FLAGS_UI: u32 = 32u; // 1 << 5

const CONSTANT_DATA_FLAGS_NONE: u32 = 0u;
const CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;
const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 2u;
const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE: u32 = 4u;
const CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX: u32 = 8u;

const MAX_FLOAT: f32 = 3.402823466e+38;

const VERTEX_ATTRIBUTE_HAS_POSITION: u32 = 0u;
const VERTEX_ATTRIBUTE_HAS_COLOR: u32 = 1u;
const VERTEX_ATTRIBUTE_HAS_NORMAL: u32 = 2u; // 1 << 1
const VERTEX_ATTRIBUTE_HAS_UV1: u32 = 4u; // 1 << 2
const VERTEX_ATTRIBUTE_HAS_UV2: u32 = 8u;  // 1 << 3
const VERTEX_ATTRIBUTE_HAS_UV3: u32 = 16u; // 1 << 4
const VERTEX_ATTRIBUTE_HAS_UV4: u32 = 32u; // 1 << 5

struct ConstantData {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    screen_width: f32,
    screen_height: f32,
    cam_fov: f32,
    flags: u32,
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

struct Mesh {
    vertices_position_offset: u32,
    vertices_attribute_offset: u32,
    vertices_attribute_layout: u32,
    material_index: i32,
    orientation: vec4<f32>,
    position: vec3<f32>,
    meshlets_offset: u32,
    scale: vec3<f32>,
    blas_index: u32,
};

struct ConeCulling {
    center: vec3<f32>,
    cone_axis_cutoff: u32,
};

struct Meshlet {
    @location(5) mesh_index: u32,
    @location(6) indices_offset: u32,
    @location(7) indices_count: u32,
    @location(8) blas_index: u32,
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

struct RuntimeVertices {
    data: array<RuntimeVertexData>,
};

struct VerticesAttributes {
    data: array<u32>,
};

struct MeshletsCulling {
    data: array<ConeCulling>,
};

struct Matrices {
    data: array<mat4x4<f32>>,
};

struct BHV {
    data: array<BHVNode>,
};

struct MeshFlags {
    data: array<u32>,
};


struct Ray {
    origin: vec3<f32>,
    t_min: f32,
    direction: vec3<f32>,
    t_max: f32,
};

struct Rays {
    data: array<Ray>,
};

struct RayPayload {
    origin: vec3<f32>,
    pixel_x: u32,
    direction: vec3<f32>,
    pixel_y: u32,
};

struct RayJob {
    index: u32,
    step: u32,
}