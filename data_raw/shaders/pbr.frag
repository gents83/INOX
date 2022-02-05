#version 450
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_nonuniform_qualifier : require

#include "common.glsl"

//Input
layout(set = 1, binding = 0) uniform sampler default_sampler;
layout(set = 1, binding = 1) uniform texture2D textures[]; 

layout(location = 0) in vec4 in_color;
layout(location = 1) in vec3 in_position;
layout(location = 2) in vec3 in_normal;
layout(location = 3) in vec3 in_tex_coord[TEXTURE_TYPE_COUNT];
layout(location = 11) in flat int in_material_index;

layout(location = 0) out vec4 frag_color;

#include "utils.glsl"

void main() {	
    uint atlas_index = getAtlasIndex(in_material_index, TEXTURE_TYPE_BASE_COLOR); 
    vec4 textureColor = texture(sampler2D(textures[atlas_index], default_sampler), in_tex_coord[TEXTURE_TYPE_BASE_COLOR].xy);
        
    vec4 out_color = textureColor * in_color;
	vec3 color_from_light = out_color.rgb;
    
	for(int i = 0; i < dynamic_data.num_lights; i++) 
	{
	    float ambient_strength = dynamic_data.light_data[i].intensity / 10000.;
	    vec3 ambient_color = dynamic_data.light_data[i].color.rgb * ambient_strength;

	    vec3 light_dir = normalize(dynamic_data.light_data[i].position - in_position);
	    
	    float diffuse_strength = max(dot(in_normal, light_dir), 0.0);
	    vec3 diffuse_color = dynamic_data.light_data[i].color.rgb * diffuse_strength;
	    vec3 view_pos = vec3(constant_data.view[3][0], constant_data.view[3][1], constant_data.view[3][2]);
	    vec3 view_dir = normalize(view_pos - in_position);

	    //Blinn-Phong
	    vec3 half_dir = normalize(view_dir + light_dir);
	    float specular_strength = pow(max(dot(in_normal, half_dir), 0.0), 32);

	    vec3 specular_color = specular_strength * dynamic_data.light_data[i].color.rgb;

	    color_from_light *= (ambient_color + diffuse_color + specular_color);
	}
    
    frag_color = vec4(color_from_light.rgb, out_color.a);
}