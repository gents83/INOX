#version 450

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 screen_size;
} pushConsts;


//Input
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;

layout(location = 4) in vec4 instanceId;
layout(location = 5) in mat4 instanceMatrix;

//Output
layout(location = 0) out vec4 outColor;

void main() {		
    gl_Position = (pushConsts.proj * pushConsts.view * instanceMatrix * vec4(inPosition.xyz, 1.));
    outColor = instanceId;
}