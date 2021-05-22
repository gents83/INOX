#version 450

layout(binding = 1) uniform sampler2DArray texSamplerArray;

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragTexCoord;
layout(location = 2) in vec4 fragDrawArea;

layout(location = 0) out vec4 outColor;

void main() {
	if (gl_FragCoord.x < fragDrawArea.x || gl_FragCoord.x > fragDrawArea.x + fragDrawArea.z 
		|| gl_FragCoord.y < fragDrawArea.y || gl_FragCoord.y > fragDrawArea.y + fragDrawArea.w) 
	{
	    discard;
	}
	if (fragTexCoord.z >= 0) 
	{
		vec4 texColor = texture(texSamplerArray, fragTexCoord);
		if(texColor.a > 0.5) 
		{
	    	outColor.rgb = texColor.rgb * fragColor.rgb;
	    	outColor.a = fragColor.a;
	    }
	    else 
	    {
	    	discard;
	    }
	    
	}
	else 
	{
		outColor = fragColor;
	}
}