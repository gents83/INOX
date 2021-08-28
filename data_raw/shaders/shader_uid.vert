#version 450

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 view_size;
	vec2 screen_size;
} pushConsts;


//Input
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;

layout(location = 4) in vec4 instanceId;
layout(location = 5) in vec3 instancePos;
layout(location = 6) in vec3 instanceRot;
layout(location = 7) in vec3 instanceScale;

//Output
layout(location = 0) out vec4 outColor;

void main() {	
	mat4 mx, my, mz;
	
	// rotate around x
	float s = sin(instanceRot.z);
	float c = cos(instanceRot.z);

	mx[0] = vec4(c, s, 0., 0.);
	mx[1] = vec4(-s, c, 0., 0.);
	mx[2] = vec4(0., 0., 1., 0.);
	mx[3] = vec4(0., 0., 0., 1.);	
	
	// rotate around y
	s = sin(instanceRot.y);
	c = cos(instanceRot.y);

	my[0] = vec4(c, 0., s, 0.);
	my[1] = vec4(0., 1., 0., 0.);
	my[2] = vec4(-s, 0., c, 0.);
	my[3] = vec4(0., 0., 0., 1.);	
	
	// rot around z
	s = sin(instanceRot.x);
	c = cos(instanceRot.x);	
	
	mz[0] = vec4(1., 0., 0., 0.);
	mz[1] = vec4(0., c, s, 0.);
	mz[2] = vec4(0., -s, c, 0.);
	mz[3] = vec4(0., 0., 0., 1.);

	mat4 rotMat = mz * my * mx;	
    mat4 transMat = mat4(	vec4(1.,0.,0.,0.),
							vec4(0.,1.,0.,0.),
                       		vec4(0.,0.,1.,0.),
                       		vec4(instancePos,1.) 
						);
    mat4 scaleMat = mat4(	vec4(instanceScale.x,0.,0.,0.),
							vec4(0.,instanceScale.y,0.,0.),
                       		vec4(0.,0.,instanceScale.z,0.),
                       		vec4(0.,0.,0.,1.) 
						);

	mat4 instanceMatrix = transMat * rotMat * scaleMat;
	
    gl_Position = (pushConsts.proj * pushConsts.view * instanceMatrix * vec4(inPosition.xy, inPosition.z, 1.));
    outColor = instanceId;
}