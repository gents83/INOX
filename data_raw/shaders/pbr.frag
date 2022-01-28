#version 450
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require

#include "common.glsl"

//Input
layout(set = 1, binding = 0) uniform sampler default_sampler;
layout(set = 1, binding = 1) uniform texture2D textures[]; 

layout(location = 0) in vec4 in_color;
layout(location = 1) in vec3 in_tex_coord[TEXTURE_TYPE_COUNT];
layout(location = 9) in flat int in_material_index;

layout(location = 0) out vec4 frag_color;

#include "utils.glsl"

void main() {	
    uint atlas_index = getAtlasIndex(in_material_index, TEXTURE_TYPE_BASE_COLOR); 
    vec4 textureColor = texture(sampler2D(textures[atlas_index], default_sampler), in_tex_coord[TEXTURE_TYPE_BASE_COLOR].xy);
        
    frag_color = textureColor * in_color;
}