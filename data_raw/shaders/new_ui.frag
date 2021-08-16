#version 450
precision highp float;

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 screen_size;
} pushConsts;

//Input
layout(binding = 1) uniform sampler2DArray texture0Sampler[8]; //texture index 0

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 inTexCoord;
layout(location = 2) flat in uint inTextureIndex;

//Output
layout(location = 0) out vec4 outColor;

void main() {
  // The texture sampler is sRGB aware, and glium already expects linear rgba output
  // so no need for any sRGB conversions here:
  outColor = inColor * texture(texture0Sampler[inTextureIndex], inTexCoord);
}