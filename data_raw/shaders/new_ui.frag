#version 450
precision highp float;

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 view_size;
	vec2 screen_size;
} pushConsts;

//Input
layout(binding = 1) uniform sampler2DArray texture0Sampler[8]; //texture index 0

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 inTexCoord;
layout(location = 2) flat in uint inTextureIndex;
layout(location = 3) in vec4 inDrawArea;

//Output
layout(location = 0) out vec4 outColor;

void main() {
	if (gl_FragCoord.x < inDrawArea.x || gl_FragCoord.x > inDrawArea.x + inDrawArea.z 
		|| gl_FragCoord.y < inDrawArea.y || gl_FragCoord.y > inDrawArea.y + inDrawArea.w) 
	{
	    discard;
	}
  // The texture sampler is sRGB aware, and glium already expects linear rgba output
  // so no need for any sRGB conversions here:
  outColor = inColor * texture(texture0Sampler[inTextureIndex], inTexCoord);
}