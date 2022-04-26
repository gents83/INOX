#extension GL_EXT_scalar_block_layout : require

#define MAX_TEXTURE_COORDS_SETS 4
#define MAX_TEXTURE_ATLAS_COUNT 16
#define MAX_NUM_MATERIALS 512
#define MAX_NUM_TEXTURES 512
#define MAX_NUM_LIGHTS 64

#define TEXTURE_TYPE_BASE_COLOR 0
#define TEXTURE_TYPE_METALLIC_ROUGHNESS 1
#define TEXTURE_TYPE_NORMAL 2
#define TEXTURE_TYPE_EMISSIVE 3
#define TEXTURE_TYPE_OCCLUSION 4
#define TEXTURE_TYPE_SPECULAR_GLOSSINESS 5
#define TEXTURE_TYPE_DIFFUSE 6
#define TEXTURE_TYPE_EMPTY_FOR_PADDING 7
#define TEXTURE_TYPE_COUNT 8

#define CONSTANT_DATA_FLAGS_NONE 0
#define CONSTANT_DATA_FLAGS_SUPPORT_SRGB 1

struct LightData
{
    vec3 position;
    uint light_type;
    vec4 color;
    float intensity;
    float range;
    float inner_cone_angle;
    float outer_cone_angle;
};

struct TextureData
{
    uint texture_index;
    uint layer_index;
    float total_width;
    float total_height;
    vec4 area;
};

struct ShaderMaterialData
{
    int textures_indices[TEXTURE_TYPE_COUNT];
    int textures_coord_set[TEXTURE_TYPE_COUNT];
    float roughness_factor;
    float metallic_factor;
    float alpha_cutoff;
    uint alpha_mode;
    vec4 base_color;
    vec4 emissive_color;
    vec4 diffuse_color;
    vec4 specular_color;
};

//Input
layout(std430, binding = 0) uniform ConstantData
{
    mat4 view;
    mat4 proj;
    vec2 screen_size;
    uint flags;
} 
constant_data;

layout(std430, binding = 1) readonly buffer DynamicData
{
    TextureData textures_data[MAX_NUM_TEXTURES];
    ShaderMaterialData material_data[MAX_NUM_MATERIALS];
    LightData light_data[MAX_NUM_LIGHTS];
} 
dynamic_data;


