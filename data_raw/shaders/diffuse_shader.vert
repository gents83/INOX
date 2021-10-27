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
layout(location = 3) in vec3 inNormal;

layout(location = 5) in mat4 instanceMatrix;

layout(location = 10) in vec4 instanceDiffuseColor;
layout(location = 11) in int instanceDiffuseTextureIndex;
layout(location = 12) in int instanceDiffuseLayerIndex;

//Output
layout(location = 0) out vec4 outColor;
layout(location = 1) out vec3 outTexCoord;
layout(location = 2) out vec3 outPosition;
layout(location = 3) out vec3 outNormal;

void main() {		
    mat4 proj_view = pushConsts.proj * pushConsts.view;
    outPosition = (instanceMatrix * vec4(inPosition.xyz, 1.)).xyz;

    mat3 normal_matrix = mat3(transpose(inverse(instanceMatrix)));
    outNormal = normal_matrix * inNormal;

    gl_Position = proj_view * vec4(outPosition.xyz, 1.);

    outColor = inColor * instanceDiffuseColor;
    outTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
}