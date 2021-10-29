#version 450
#extension GL_GOOGLE_include_directive : require

#include "common.glsl"

layout(binding = 1) uniform sampler2DArray texture0Sampler[8]; //texture index 0

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 inPosition;
layout(location = 2) in vec3 inNormal;
layout(location = 4) in flat uint inTexIdx[TEXTURE_TYPE_COUNT];
layout(location = 12) in vec3 inTexCoords[TEXTURE_TYPE_COUNT];

//Output
layout(location = 0) out vec4 outColor;


void main() {
	if (inTexCoords[TEXTURE_TYPE_BASE_COLOR].z >= 0) 
	{
		vec4 texColor = texture(texture0Sampler[inTexIdx[TEXTURE_TYPE_BASE_COLOR]], inTexCoords[TEXTURE_TYPE_BASE_COLOR]);
		if(texColor.a > 0.01) 
		{
	    	outColor.rgb = texColor.rgb * inColor.rgb;
	    	outColor.a = inColor.a;
	    }
	    else 
	    {
	    	discard;
	    }
	    
	}
	else 
	{
		outColor = inColor;
	}		
	
	vec3 color_from_light = outColor.rgb;

	for(int i = 0; i < uniforms.num_lights; i++) 
	{
	    float ambient_strength = uniforms.light_data[i].intensity / 10000.;
	    vec3 ambient_color = uniforms.light_data[i].color.rgb * ambient_strength;

	    vec3 normal = inNormal;
	    vec3 light_dir = normalize(uniforms.light_data[i].position - inPosition);
	    
	    float diffuse_strength = max(dot(normal, light_dir), 0.0);
	    vec3 diffuse_color = uniforms.light_data[i].color.rgb * diffuse_strength;

	    vec3 view_pos = vec3(globals.view[3][0], globals.view[3][1], globals.view[3][2]);
	    vec3 view_dir = normalize(view_pos - inPosition);

	    //Blinn-Phong
	    vec3 half_dir = normalize(view_dir + light_dir);
	    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 32);

	    vec3 specular_color = specular_strength * uniforms.light_data[i].color.rgb;

	    color_from_light *= (ambient_color + diffuse_color + specular_color);
	}

    outColor = vec4(color_from_light, outColor.a);
}