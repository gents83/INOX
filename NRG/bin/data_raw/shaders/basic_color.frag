#version 450

//Input
layout(location = 0) in vec4 inColor;

//Output
layout(location = 0) out vec4 outColor;


void main() {
	outColor = inColor;
}