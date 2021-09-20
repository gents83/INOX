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

layout(location = 8) in vec4 instanceDrawArea;
layout(location = 10) in int instanceDiffuseTextureIndex;
layout(location = 11) in int instanceDiffuseLayerIndex;

//Output
layout(location = 0) out vec4 outColor;
layout(location = 1) out vec3 outTexCoord;
layout(location = 2) out uint outTextureIndex;

vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, vec3(cutoff));
}
vec4 linear_from_srgba(vec4 srgba) {
    return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main() {
  gl_Position =
      vec4(2.0 * inPosition.x / pushConsts.screen_size.x - 1.0,
           2.0 * inPosition.y / pushConsts.screen_size.y - 1.0, 
           1000. - inPosition.z, 
           1.0);
  // egui encodes vertex colors in gamma spaces, so we must decode the colors here:
  outColor = inColor;  
  outTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
  outTextureIndex = instanceDiffuseTextureIndex;
}