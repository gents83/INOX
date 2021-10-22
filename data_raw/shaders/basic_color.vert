#version 450

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 screen_size;
} pushConsts;


//Input
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(location = 5) in mat4 instanceMatrix;

layout(location = 10) in vec4 instanceDiffuseColor;
layout(location = 11) in int instanceDiffuseTextureIndex;
layout(location = 12) in int instanceDiffuseLayerIndex;

//Output
layout(location = 0) out vec4 outColor;

void main() {		
    gl_Position = (pushConsts.proj * pushConsts.view * instanceMatrix * vec4(inPosition.xyz, 1.));
    outColor = inColor * instanceDiffuseColor;
}