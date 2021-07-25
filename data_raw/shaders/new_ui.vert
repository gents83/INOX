#version 450
precision highp float;

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

layout(location = 4) in vec3 instancePos;
layout(location = 6) in vec3 instanceScale;
layout(location = 7) in vec4 instanceDrawArea;

layout(location = 8) in vec4 instanceDiffuseColor;
layout(location = 9) in int instanceDiffuseTextureIndex;
layout(location = 10) in int instanceDiffuseLayerIndex;

layout(location = 11) in vec4 instanceOutlineColor;

//Output
layout(location = 0) out vec4 outColor;
layout(location = 1) out vec3 outTexCoord;

float ui_scale = 2.;

void main() {
  gl_Position =
      vec4(2.0 * inPosition.x / pushConsts.screen_size.x - 1.0,
           2.0 * inPosition.y / pushConsts.screen_size.y - 1.0, 
           0.0, 
           1.0);
  // egui encodes vertex colors in gamma spaces, so we must decode the colors here:
  outColor = inColor;  
  outTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
}