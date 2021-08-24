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

//Output
layout(location = 0) out vec4 outColor;
layout(location = 1) out vec3 outNearPoint;
layout(location = 2) out vec3 outFarPoint;

vec3 UnprojectPoint(vec3 position, mat4 view, mat4 projection) {
    mat4 viewInv = inverse(view);
    mat4 projInv = inverse(projection);
    vec4 unprojectedPoint =  viewInv * projInv * vec4(position, 1.0);
    return unprojectedPoint.xyz / unprojectedPoint.w;
}

void main() {	
    outNearPoint = UnprojectPoint(vec3(inPosition.xy, 0.0), pushConsts.view, pushConsts.proj).xyz; // unprojecting on the near plane
    outFarPoint = UnprojectPoint(vec3(inPosition.xy, 1.0), pushConsts.view, pushConsts.proj).xyz; // unprojecting on the far plane

    outColor = inColor;
    gl_Position = vec4(inPosition.xyz, 1.0);
}