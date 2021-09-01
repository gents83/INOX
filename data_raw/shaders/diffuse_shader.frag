#version 450

layout(std140, push_constant) uniform PushConsts {
    mat4 view;
    mat4 proj;
	vec2 screen_size;
} pushConsts;

//Input
layout(binding = 1) uniform sampler2DArray texSamplerArray; //texture index 0

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 inTexCoord;

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
}