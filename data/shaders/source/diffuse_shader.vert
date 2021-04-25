#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;

layout(location = 4) in mat4 instanceMatrix;
layout(location = 8) in vec4 instanceDiffuseColor;
layout(location = 9) in int instanceDiffuseTextureIndex;
layout(location = 10) in int instanceDiffuseLayerIndex;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 fragTexCoord;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

void main() {	
    gl_Position = ubo.proj * ubo.view * instanceMatrix * vec4(inPosition, 1.0);

    fragColor = inColor * instanceDiffuseColor;
    fragTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
}