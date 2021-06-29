#version 450

//Input
layout (triangles) in;

layout(location = 0) in vec4 inColor[];
layout(location = 1) in vec3 inTexCoord[];
layout(location = 2) in vec4 inDrawArea[];
layout(location = 3) in vec4 inOutlineColor[];

//Output
layout (triangle_strip, max_vertices = 18) out;

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec3 outTexCoord;
layout(location = 2) out vec4 outDrawArea;
layout(location = 3) out vec4 outOutlineColor;
layout(location = 4) out vec3 outBarycentricCoord;

struct VertData {
	vec4 position;
	vec4 color;
	vec3 texCoord;
	vec4 drawArea;
	vec4 outlineColor;
	vec3 barycentricCoord;
};

#define ROUNDED_CORNER_RADIUS 0.0

vec3 GetBarycentricCoord(int index) 
{
	if (index == 0) {
		return vec3(1,0,0);
	}
	else if (index == 1) {
		return vec3(0,1,0);
	}
	else {
		return vec3(0,0,1);
	}
}

vec4 InterpolateVec4(vec4 v1, vec4 v2, bool shouldNormalize, float l) 
{
	vec4 dir = (v2 - v1);
	if (shouldNormalize)
	{
		dir = normalize(dir);	
	}
	return v1 + dir * l;
}

vec3 InterpolateVec3(vec3 v1, vec3 v2, bool shouldNormalize, float l) 
{
	vec3 dir = (v2 - v1);
	if (shouldNormalize)
	{
		dir = normalize(dir);	
	}
	return v1 + dir * l;
}

vec2 InterpolateVec2(vec2 v1, vec2 v2, bool shouldNormalize, float l) 
{
	vec2 dir = (v2 - v1);
	if (shouldNormalize)
	{
		dir = normalize(dir);	
	}
	return v1 + dir * l;
}


VertData ComputeVertData(int i) 
{
	VertData data;
	data.position = gl_in[i].gl_Position;
	data.color = inColor[i];
	data.texCoord = inTexCoord[i];
	data.drawArea = inDrawArea[i];
	data.outlineColor = inOutlineColor[i];
	data.barycentricCoord = GetBarycentricCoord(i);
	return data;
}

VertData InterpolateData(VertData v1, VertData v2, bool shouldNormalize, float l) 
{
	VertData data;
	data.position = InterpolateVec4(v1.position, v2.position, shouldNormalize, l);
	data.color = InterpolateVec4(v1.color, v2.color, false, l);
	data.texCoord = vec3(InterpolateVec2(v1.texCoord.xy, v2.texCoord.xy, shouldNormalize, l), v1.texCoord.z);
	data.drawArea = v1.drawArea;
	data.outlineColor = v1.outlineColor;
	data.barycentricCoord = InterpolateVec3(v1.barycentricCoord, v2.barycentricCoord, false, l);
	return data;
}

void Publish(VertData v)
{
	gl_Position = v.position;
	outColor = v.color;
	outTexCoord = v.texCoord;
	outDrawArea = v.drawArea;
	outOutlineColor = v.outlineColor;
	outBarycentricCoord = v.barycentricCoord;
	EmitVertex();
}

void main(void)
{	
	VertData v0 = ComputeVertData(0);
	VertData v1 = ComputeVertData(1);
	VertData v2 = ComputeVertData(2);
	VertData v3 = InterpolateData(v0, v2, false, 0.5);
	VertData v4 = InterpolateData(v1, v0, true, ROUNDED_CORNER_RADIUS);
	VertData v5 = InterpolateData(v1, v2, true, ROUNDED_CORNER_RADIUS);
	VertData v6 = InterpolateData(v1, v3, true, ROUNDED_CORNER_RADIUS * 0.5);
	VertData v7 = InterpolateData(v0, v1, true, ROUNDED_CORNER_RADIUS);
	VertData v8 = InterpolateData(v2, v1, true, ROUNDED_CORNER_RADIUS);
	VertData v9 = InterpolateData(v0, v2, true, ROUNDED_CORNER_RADIUS * 0.5);
	VertData v10 = InterpolateData(v2, v0, true, ROUNDED_CORNER_RADIUS * 0.5);
	
	Publish(v9); Publish(v6); Publish(v10);
	
	Publish(v9); Publish(v7); Publish(v4);
	Publish(v4); Publish(v6); Publish(v9);  
	
	Publish(v6); Publish(v5); Publish(v8);
	Publish(v8); Publish(v10); Publish(v6);  
	
	EndPrimitive();
}
