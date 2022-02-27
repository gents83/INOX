#version 450
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require

#include "common.glsl"

//Input
layout(set = 1, binding = 0) uniform sampler default_sampler;
layout(set = 1, binding = 1) uniform texture2D texture_1; 
layout(set = 1, binding = 2) uniform texture2D texture_2; 
layout(set = 1, binding = 3) uniform texture2D texture_3; 
layout(set = 1, binding = 4) uniform texture2D texture_4; 
layout(set = 1, binding = 5) uniform texture2D texture_5; 
layout(set = 1, binding = 6) uniform texture2D texture_6; 
layout(set = 1, binding = 7) uniform texture2D texture_7; 
layout(set = 1, binding = 8) uniform texture2D texture_8; 
layout(set = 1, binding = 9) uniform texture2D texture_9; 
layout(set = 1, binding = 10) uniform texture2D texture_10; 
layout(set = 1, binding = 11) uniform texture2D texture_11; 
layout(set = 1, binding = 12) uniform texture2D texture_12; 
layout(set = 1, binding = 13) uniform texture2D texture_13; 
layout(set = 1, binding = 14) uniform texture2D texture_14; 
layout(set = 1, binding = 15) uniform texture2D texture_15; 
layout(set = 1, binding = 16) uniform texture2D texture_16; 

layout(location = 0) in vec4 in_color;
layout(location = 1) in vec3 in_tex_coord[TEXTURE_TYPE_COUNT];
layout(location = 9) in flat int in_material_index;

layout(location = 0) out vec4 frag_color;

#include "utils.glsl"
#include "frag_utils.glsl"

void main() {	
    vec4 textureColor = getTextureColor(TEXTURE_TYPE_BASE_COLOR);
    frag_color = textureColor * in_color;
}