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
layout(location = 2) out vec4 outDrawArea;
layout(location = 3) out vec4 outOutlineColor;

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


mat4 CreateInstanceMatrix(vec3 position, vec3 rotation, vec3 scale) {
    
	mat4 mx, my, mz;
	
	// rotate around x
	float s = sin(rotation.z);
	float c = cos(rotation.z);

	mx[0] = vec4(c, s, 0., 0.);
	mx[1] = vec4(-s, c, 0., 0.);
	mx[2] = vec4(0., 0., 1., 0.);
	mx[3] = vec4(0., 0., 0., 1.);	
	
	// rotate around y
	s = sin(rotation.y);
	c = cos(rotation.y);

	my[0] = vec4(c, 0., s, 0.);
	my[1] = vec4(0., 1., 0., 0.);
	my[2] = vec4(-s, 0., c, 0.);
	my[3] = vec4(0., 0., 0., 1.);	
	
	// rot around z
	s = sin(rotation.x);
	c = cos(rotation.x);	
	
	mz[0] = vec4(1., 0., 0., 0.);
	mz[1] = vec4(0., c, s, 0.);
	mz[2] = vec4(0., -s, c, 0.);
	mz[3] = vec4(0., 0., 0., 1.);

	mat4 rotMat = mz * my * mx;	
    mat4 transMat = mat4(	vec4(1.,0.,0.,0.),
							vec4(0.,1.,0.,0.),
                       		vec4(0.,0.,1.,0.),
                       		vec4(position,1.) 
						);
    mat4 scaleMat = mat4(	vec4(scale.x,0.,0.,0.),
							vec4(0.,scale.y,0.,0.),
                       		vec4(0.,0.,scale.z,0.),
                       		vec4(0.,0.,0.,1.) 
						);

	return transMat * rotMat * scaleMat;
}


void main() {	
	mat4 ortho_proj = CreateOrthoMatrix(0., pushConsts.screen_size.x, 0., pushConsts.screen_size.y, 0., 1000.);

    mat4 instanceMatrix = mat4(	vec4(instanceScale.x,0.,0.,0.),
							vec4(0.,instanceScale.y,0.,0.),
                       		vec4(0.,0.,1.,0.),
                       		vec4(instancePos,1.) 
						);
	
	vec2 outlineOffset = ((2 * inNormal.xy * instanceOutlineColor.a) - 1) / pushConsts.screen_size.xy;


    gl_Position = (ortho_proj * instanceMatrix * vec4((inPosition.xy + outlineOffset), inPosition.z-500., 1.));

    outColor = inColor * instanceDiffuseColor;
    outTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
    outDrawArea = instanceDrawArea;
    outOutlineColor = instanceOutlineColor;
}