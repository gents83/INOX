#version 450

//Input
layout (triangles) in;

layout(location = 0) in vec4 inColor[];
layout(location = 1) in vec3 inTexCoord[];
layout(location = 2) in vec4 inDrawArea[];
layout(location = 3) in vec4 inOutlineColor[];

//Output
layout (triangle_strip, max_vertices = 3) out;

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec3 outTexCoord;
layout(location = 2) out vec4 outDrawArea;
layout(location = 3) out vec4 outOutlineColor;

layout(location = 4) out vec3 outBarycentricCoord;

void main(void)
{	
    for(int i = 0; i < gl_in.length(); i++)
    {
		outColor = inColor[i];
		outTexCoord = inTexCoord[i];
		outDrawArea = inDrawArea[i];
		outOutlineColor = inOutlineColor[i];
		if (i==0) {
			outBarycentricCoord = vec3(1,0,0);
        }
		else if (i==1) {
			outBarycentricCoord = vec3(0,1,0);
        }
		else {
			outBarycentricCoord = vec3(0,0,1);
		}
		gl_Position = gl_in[i].gl_Position;
        EmitVertex();
    }

	EndPrimitive();
}
