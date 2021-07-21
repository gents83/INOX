#version 450
precision highp float;

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 screen_size;
} pushConsts;

//Input
layout(binding = 1) uniform sampler2DArray texSamplerArray; //texture index 0

layout(location = 0) in vec2 inCoords;
layout(location = 1) in vec4 inRect;
layout(location = 2) in vec4 inColor;
layout(location = 3) in vec3 inTexCoord;

//Output
layout(location = 0) out vec4 outColor;

float roundness = 0.1;

void main() {
	vec2 rect_size = vec2(inRect.z-inRect.x, inRect.w-inRect.y); 
	vec2 pointOnSegmentLine = vec2( clamp(inCoords.x, inRect.x + rect_size.x * roundness, 
													  inRect.z - rect_size.x * roundness),
									clamp(inCoords.y, inRect.y + rect_size.y * roundness, 
													  inRect.w - rect_size.y * roundness) );
	float dist = distance(inCoords, pointOnSegmentLine) * 10.;	
	outColor = step(dist, roundness) * inColor;	
}