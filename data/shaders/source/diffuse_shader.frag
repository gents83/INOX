#version 450

//Input
layout(binding = 1) uniform sampler2DArray texSamplerArray;

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec3 inTexCoord;
layout(location = 2) in vec4 inDrawArea;
layout(location = 3) in vec4 inOutlineColor;

layout(location = 4) in vec3 inBarycentricCoord;

//Output
layout(location = 0) out vec4 outColor;

void main() {
	if (gl_FragCoord.x < inDrawArea.x || gl_FragCoord.x > inDrawArea.x + inDrawArea.z 
		|| gl_FragCoord.y < inDrawArea.y || gl_FragCoord.y > inDrawArea.y + inDrawArea.w) 
	{
	    discard;
	}

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

	if (inOutlineColor.a >= 0.01) 
	{		
		vec2 screenParams = vec2(2700, 1574);
		vec2 a = vec2(1 / screenParams.x, 1 / screenParams.y);
		vec3 t = vec3(a.x, a.y, min(a.x, a.y)) * inOutlineColor.a;

		vec3 excludeInternalEdge = vec3(0.0, 1.0, 0.0);
		vec3 s = step(t * 2, inBarycentricCoord + excludeInternalEdge);
		float v = 1. - min(s.r, min(s.g, s.b));
		
		outColor = mix(outColor, vec4(inOutlineColor.rgb, 1.0), v);
	}
		
}