#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;

layout(location = 4) in mat4 instanceMatrix;
layout(location = 8) in int instanceDiffuseTextureIndex;
layout(location = 9) in int instanceDiffuseLayerIndex;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 fragTexCoord;

void main() {
    gl_Position = instanceMatrix * vec4(inPosition, 1.0);
    fragColor = inColor;
    fragTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
}