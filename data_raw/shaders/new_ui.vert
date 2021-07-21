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
layout(location = 0) out vec2 outVertCoord;
layout(location = 1) out vec4 outRectangle;
layout(location = 2) out vec4 outColor;
layout(location = 3) out vec3 outTexCoord;

mat4 CreateOrthoMatrix(float left_plane, float right_plane, float top_plane, float bottom_plane, float near_plane, float far_plane) {
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
	mat4 ortho_proj = CreateOrthoMatrix(0., pushConsts.screen_size.x, 0., pushConsts.screen_size.y, 0., 1000.);

    mat4 instanceMatrix = mat4(	vec4(instanceScale.x,0.,0.,0.),
								vec4(0.,instanceScale.y,0.,0.),
                       			vec4(0.,0.,1.,0.),
                       			vec4(instancePos,1.) 
							);
	mat4 mvp = ortho_proj * instanceMatrix;
    vec4 vertex_position = (mvp * vec4((inPosition.xy), inPosition.z-1000., 1.));
    vec4 min_in_screen_space = (mvp * vec4(0., 0., inPosition.z-1000., 1.));
    vec4 max_in_screen_space = (mvp * vec4(1., 1., inPosition.z-1000., 1.));

    gl_Position = vertex_position;
	outVertCoord = vertex_position.xy;
    outRectangle = vec4(min_in_screen_space.xy, max_in_screen_space.xy);
    outColor = inColor * instanceDiffuseColor;
    outTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
	
}