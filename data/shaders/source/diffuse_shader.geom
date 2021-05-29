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
	vec3 param = vec3(0.0, 0.0, 0.0);

	float edgeA = length(gl_in[0].gl_Position - gl_in[1].gl_Position);
	float edgeB = length(gl_in[1].gl_Position - gl_in[2].gl_Position);
	float edgeC = length(gl_in[2].gl_Position - gl_in[0].gl_Position);
	
    if(edgeA > edgeB && edgeA > edgeC)
        param.z = 1.;
    else if (edgeB > edgeC && edgeB > edgeA)
        param.x = 1.;
    else
        param.y = 1.;

    int i;
    for(i = 0; i < gl_in.length(); i++)
    {
		outColor = inColor[i];
		outTexCoord = inTexCoord[i];
		outDrawArea = inDrawArea[i];
		outOutlineColor = inOutlineColor[i];
		if (i==0) {
			outBarycentricCoord = vec3(1,0,0) + param;
        }
		else if (i==1) {
			outBarycentricCoord = vec3(0,1,0) + param;
        }
		else {
			outBarycentricCoord = vec3(0,0,1) + param;
		}
		gl_Position = gl_in[i].gl_Position;
        EmitVertex();
    }

	EndPrimitive();
}
