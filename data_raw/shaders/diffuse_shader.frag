#version 450

#define MAX_NUM_LIGHTS 32

struct LightData {
    vec3 position;
    uint light_type; //act as padding too
    vec4 color;
    float intensity;
    float range;
    float inner_cone_angle;
    float outer_cone_angle;
};

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 screen_size;
} pushConsts;

//Input

layout(binding = 0) uniform UniformData {
    uint num_lights;
    uint _padding[3];
    LightData light_data[MAX_NUM_LIGHTS];
} uniform_data;

layout(binding = 1) uniform sampler2DArray texSamplerArray; //texture index 0

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 inTexCoord;
layout(location = 2) in vec3 inPosition;
layout(location = 3) in vec3 inNormal;

//Output
layout(location = 0) out vec4 outColor;


void main() {
	if (inTexCoord.z >= 0) 
	{
		vec4 texColor = texture(texSamplerArray, inTexCoord);
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

	for(int i = 0; i < uniform_data.num_lights; i++) 
	{
	    float ambient_strength = uniform_data.light_data[i].intensity / 10000.;
	    vec3 ambient_color = uniform_data.light_data[i].color.rgb * ambient_strength;

	    vec3 normal = inNormal;
	    vec3 light_dir = normalize(uniform_data.light_data[i].position - inPosition);
	    
	    float diffuse_strength = max(dot(normal, light_dir), 0.0);
	    vec3 diffuse_color = uniform_data.light_data[i].color.rgb * diffuse_strength;

	    vec3 view_pos = vec3(pushConsts.view[0][3], pushConsts.view[1][3], pushConsts.view[2][3]);
	    vec3 view_dir = normalize(view_pos - inPosition);

	    //Blinn-Phong
	    vec3 half_dir = normalize(view_dir + light_dir);
	    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 8);

	    vec3 specular_color = specular_strength * uniform_data.light_data[i].color.rgb;

	    color_from_light *= (ambient_color + diffuse_color + specular_color);
	}

    outColor = vec4(color_from_light, outColor.a);
}