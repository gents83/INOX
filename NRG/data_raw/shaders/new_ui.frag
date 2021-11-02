#version 450
#extension GL_GOOGLE_include_directive : require

#include "common.glsl"

//Input
layout(binding = 1) uniform sampler2DArray texture0Sampler[8]; //texture index 0

layout(location = 0) in vec4 inColor;
layout(location = 1) in flat uint inTexIdx[TEXTURE_TYPE_COUNT];
layout(location = 9) in vec3 inTexCoords[TEXTURE_TYPE_COUNT];

//Output
layout(location = 0) out vec4 outColor;

void main() {	
    vec4 textureColor = texture(texture0Sampler[inTexIdx[TEXTURE_TYPE_BASE_COLOR]], inTexCoords[TEXTURE_TYPE_BASE_COLOR]);
    outColor = textureColor * inColor;
}