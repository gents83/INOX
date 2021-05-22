#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;

layout(location = 4) in vec3 instancePos;
layout(location = 5) in vec3 instanceRot;
layout(location = 6) in vec3 instanceScale;
layout(location = 7) in vec4 instanceDrawArea;
layout(location = 8) in vec4 instanceDiffuseColor;
layout(location = 9) in int instanceDiffuseTextureIndex;
layout(location = 10) in int instanceDiffuseLayerIndex;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec3 fragTexCoord;
layout(location = 2) out vec4 fragDrawArea;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

void main() {	
	mat4 mx, my, mz;
	
	// rotate around x
	float s = sin(instanceRot.x);
	float c = cos(instanceRot.x);

	mx[0] = vec4(c, s, 0.0, 0.0);
	mx[1] = vec4(-s, c, 0.0, 0.0);
	mx[2] = vec4(0.0, 0.0, 1.0, 0.0);
	mx[3] = vec4(0.0, 0.0, 0.0, 1.0);	
	
	// rotate around y
	s = sin(instanceRot.y);
	c = cos(instanceRot.y);

	my[0] = vec4(c, 0.0, s, 0.0);
	my[1] = vec4(0.0, 1.0, 0.0, 0.0);
	my[2] = vec4(-s, 0.0, c, 0.0);
	my[3] = vec4(0.0, 0.0, 0.0, 1.0);	
	
	// rot around z
	s = sin(instanceRot.z);
	c = cos(instanceRot.z);	
	
	mz[0] = vec4(1.0, 0.0, 0.0, 0.0);
	mz[1] = vec4(0.0, c, s, 0.0);
	mz[2] = vec4(0.0, -s, c, 0.0);
	mz[3] = vec4(0.0, 0.0, 0.0, 1.0);

	mat4 rotMat = mz * my * mx;	
	vec4 pos = vec4((inPosition.xyz * instanceScale) + instancePos, 1.0) * rotMat;

    gl_Position = ubo.proj * ubo.view * pos;

    fragColor = inColor * instanceDiffuseColor;
    fragTexCoord = vec3(inTexCoord, instanceDiffuseLayerIndex);
    fragDrawArea = instanceDrawArea;
}