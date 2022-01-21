let MAX_TEXTURE_ATLAS_COUNT: u32 = 16u;
let MAX_NUM_LIGHTS: u32 = 64u;
let MAX_NUM_TEXTURES: u32 = 512u;
let MAX_NUM_MATERIALS: u32 = 512u;

let TEXTURE_TYPE_BASE_COLOR: u32 = 0u;
let TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;
let TEXTURE_TYPE_NORMAL: u32 = 2u;
let TEXTURE_TYPE_EMISSIVE: u32 = 3u;
let TEXTURE_TYPE_OCCLUSION: u32 = 4u;
let TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;
let TEXTURE_TYPE_DIFFUSE: u32 = 6u;
let TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;
let TEXTURE_TYPE_COUNT: u32 = 8u;

struct ConstantData {
    view: mat4x4<f32>;
    proj: mat4x4<f32>;
    screen_width: f32;
    screen_height: f32;
    num_textures: u32;
    num_materials: u32;
    num_lights: u32;
    padding: u32;
};

struct LightData {
    position: vec3<f32>;
    light_type: u32;
    color: vec4<f32>;
    intensity: f32;
    range: f32;
    inner_cone_angle: f32;
    outer_cone_angle: f32;
};

struct TextureData {
    texture_index: u32;
    layer_index: u32;
    total_width: u32;
    total_height: u32;
    area: vec4<f32>;
};

struct ShaderMaterialData {
    textures_indices: array<i32, TEXTURE_TYPE_COUNT>;
    textures_coord_set: array<u32, TEXTURE_TYPE_COUNT>;
    roughness_factor: f32;
    metallic_factor: f32;
    alpha_cutoff: f32;
    alpha_mode: u32;
    base_color: vec4<f32>;
    emissive_color: vec4<f32>;
    diffuse_color: vec4<f32>;
    specular_color: vec4<f32>;
};

struct DynamicData {
    lights: array<LightData, MAX_NUM_LIGHTS>;
    textures: array<TextureData, MAX_NUM_TEXTURES>;
    materials: array<ShaderMaterialData, MAX_NUM_MATERIALS>;
};


[[group(0), binding(0)]]
var<uniform> constant_data: ConstantData;
[[group(0), binding(1)]]
var<storage, read> dynamic_data: DynamicData;

struct InstanceInput {
    //[[builtin(instance_index)]] index: u32;
    [[location(8)]] id: vec4<f32>;
    [[location(9)]] model_matrix_0: vec4<f32>;
    [[location(10)]] model_matrix_1: vec4<f32>;
    [[location(11)]] model_matrix_2: vec4<f32>;
    [[location(12)]] model_matrix_3: vec4<f32>;
    [[location(13)]] material_index: i32;
};

struct VertexInput {
    //[[builtin(vertex_index)]] index: u32;
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tangent: vec3<f32>;
    [[location(3)]] color: vec4<f32>;
    [[location(4)]] tex_coords_0: vec2<f32>;
    [[location(5)]] tex_coords_1: vec2<f32>;
    [[location(6)]] tex_coords_2: vec2<f32>;
    [[location(7)]] tex_coords_3: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] tex_coords_0: vec2<f32>;
    [[location(2)]] material_index: i32;
};

[[stage(vertex)]]
fn vs_main(
    v: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let instance_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.color = v.color;
    out.tex_coords_0 = v.tex_coords_0;
    out.material_index = instance.material_index;
    out.clip_position = constant_data.proj * constant_data.view * instance_matrix * vec4<f32>(v.position, 1.0);
    return out;
}

[[group(1), binding(0)]]
var default_sampler: sampler;
[[group(1), binding(1)]]
var texture_array: texture_2d_array<f32>;

[[stage(fragment)]]
fn fs_main(v: VertexOutput) -> [[location(0)]] vec4<f32> {     
    var diffuse_index = 0;
    if (v.material_index >= 0) {
        diffuse_index = dynamic_data.materials[v.material_index].textures_indices[TEXTURE_TYPE_BASE_COLOR];
    }
    let area = dynamic_data.textures[diffuse_index].area;
    let index = i32(dynamic_data.textures[diffuse_index].layer_index);
    let size = vec2<f32>(f32(dynamic_data.textures[diffuse_index].total_width), f32(dynamic_data.textures[diffuse_index].total_height));
    let tex_coords = vec2<f32>(
                (area.x + (v.tex_coords_0.x * area.z)) / size.x,
                (area.y + (v.tex_coords_0.y * area.w)) / size.y);
    let t = textureSample(texture_array, default_sampler, tex_coords.xy, index);
    if (diffuse_index >= 0) {
        return t * v.color;
    }
    return v.color;
    
}