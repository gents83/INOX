#version 450
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require

#include "common.glsl"

//Input
layout(set = 1, binding = 0) uniform sampler default_sampler;
layout(set = 1, binding = 1) uniform sampler depth_sampler;
layout(set = 1, binding = 2) uniform texture2D texture_array[MAX_TEXTURE_ATLAS_COUNT];

layout(location = 0) in vec4 in_color;
layout(location = 1) in flat int in_material_index;
layout(location = 2) in vec3 in_tex_coord;

layout(location = 0) out vec4 frag_color;

#include "utils.glsl"

vec4 getTextureColor(uint texture_type) 
{
    uint texture_index = getAtlasIndex(in_material_index, texture_type);     
    return texture(
        sampler2D(texture_array[texture_index], default_sampler), 
        in_tex_coord.xy
    ); 
}

void main() {	    
    vec4 textureColor = getTextureColor(TEXTURE_TYPE_BASE_COLOR);
    frag_color = in_color * textureColor; 
}