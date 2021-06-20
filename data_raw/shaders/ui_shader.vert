#version 450

layout(push_constant) uniform PushConsts {
	vec2 screen_size;
    mat4 view;
    mat4 proj;
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
layout(location = 2) out vec4 outDrawArea;
layout(location = 3) out vec4 outOutlineColor;

mat4 create_orto_matrix(float left_plane, float right_plane, float top_plane, float bottom_plane, float near_plane, float far_plane) {
	return mat4(
      2.0f / (right_plane - left_plane),
      0.0f,
      0.0f,
      0.0f,

      0.0f,
      2.0f / (bottom_plane - top_plane),
      0.0f,
      0.0f,

      0.0f,
      0.0f,
      1.0f / (near_plane - far_plane),
      0.0f,

      -(right_plane + left_plane) / (right_plane - left_plane),
      -(bottom_plane + top_plane) / (bottom_plane - top_plane),
      near_plane / (near_plane - far_plane),
      1.0f
    );
}

void main() {	
	mat4 ortho_proj = create_orto_matrix(0., pushConsts.screen_size.x, 0., pushConsts.screen_size.y, -100., 0.);

    mat4 instanceMatrix = mat4(	vec4(instanceScale.x,0.,0.,0.),
							vec4(0.,instanceScale.y,0.,0.),
                       		vec4(0.,0.,1.,0.),
                       		vec4(instancePos,1.) 
						);
	
	vec2 outlineOffset = ((2 * inNormal.xy * instanceOutlineColor.a) - 1) / pushConsts.screen_size.xy;


    gl_Position = (ortho_proj * instanceMatrix * vec4((inPosition.xy + outlineOffset), inPosition.z, 1.));

    outColor = inColor * instanceDiffuseColor;
    outTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
    outDrawArea = instanceDrawArea;
    outOutlineColor = instanceOutlineColor;
}