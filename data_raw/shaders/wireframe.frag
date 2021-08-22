#version 450

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 view_size;
	vec2 screen_size;
} pushConsts;

//Input
layout(location = 0) in vec4 inColor;

//Output
layout(location = 0) out vec4 outColor;


void main() {
	outColor = inColor;
}