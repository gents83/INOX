#version 450

layout(binding = 1) uniform sampler2DArray texSamplerArray;

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main() {
	if (fragTexCoord.z >= 0) {
		vec4 texColor = texture(texSamplerArray, fragTexCoord);
		if(texColor.a > 0.5) {
	    	outColor.rgb = texColor.rgb * fragColor.rgb;
	    	outColor.a = fragColor.a;
	    }
	    else 
	    	discard;
	    
	}
	else {
		outColor = fragColor;
	}
}