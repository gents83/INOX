#version 450


layout(push_constant) uniform PushConsts {
	vec2 screen_size;
    mat4 view;
    mat4 proj;
} pushConsts;

//Input
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;

//Output
layout(location = 0) out vec4 outColor;
layout(location = 1) out vec3 outNearPoint;
layout(location = 2) out vec3 outFarPoint;
layout(location = 3) out mat4 outView;
layout(location = 7) out mat4 outProj;

float deg_to_rad(float deg) {
    return deg * 3.14 / 180;
}

mat4 CreatePerpectiveMatrix(float aspect_ratio, float field_of_view, float near_plane, float far_plane) {
	float f = 1.0f / tan( 0.5 * field_of_view );

    return mat4(      
      f / aspect_ratio,
      0.0f,
      0.0f,
      0.0f,

      0.0f,
      -f,
      0.0f,
      0.0f,

      0.0f,
      0.0f,
      far_plane / (near_plane - far_plane),
      -1.0f,
      
      0.0f,
      0.0f,
      (near_plane * far_plane) / (near_plane - far_plane),
      0.0f
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

	return rotMat * scaleMat * transMat;
}

vec3 UnprojectPoint(vec3 position, mat4 view, mat4 projection) {
    mat4 viewInv = inverse(view);
    mat4 projInv = inverse(projection);
    vec4 unprojectedPoint =  viewInv * projInv * vec4(position, 1.0);
    return unprojectedPoint.xyz / unprojectedPoint.w;
}

void main() {	

    mat4 proj = CreatePerpectiveMatrix(
                    pushConsts.screen_size.x/pushConsts.screen_size.y, 
                    deg_to_rad(45.),
                    0.001,
                    1000.0);
    proj[1][1] *= -1.0f;
    mat4 view = CreateInstanceMatrix(
                    vec3(0., 2., -2.), 
                    vec3(-deg_to_rad(45.),deg_to_rad(0.), deg_to_rad(0.)), 
                    vec3(1.,1., 1.));
    
    gl_Position = vec4(inPosition.xyz, 1.0);

    outColor = inColor;
    outNearPoint = UnprojectPoint(vec3(inPosition.xy, 0.0), view, proj).xyz; // unprojecting on the near plane
    outFarPoint = UnprojectPoint(vec3(inPosition.xy, 1.0), view, proj).xyz; // unprojecting on the far plane
    outView = view;
    outProj = proj;
}