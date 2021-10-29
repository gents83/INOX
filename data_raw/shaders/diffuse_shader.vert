#version 450
#extension GL_GOOGLE_include_directive : require

#include "common.glsl"

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec3 vertex_tangent;
layout(location = 3) in vec4 vertex_color;
layout(location = 4) in vec2 vertex_tex_coord[MAX_TEXTURE_COORDS_SETS];

layout(location = 9) in mat4 instance_matrix;
layout(location = 14) in int  instance_material_index;

//Output
layout(location = 0) out vec4 outColor;
layout(location = 1) out vec3 outPosition;
layout(location = 2) out vec3 outNormal;
layout(location = 3) out int  outMaterialIndex;
layout(location = 4) out uint  outTexIdx[TEXTURE_TYPE_COUNT];
layout(location = 12) out vec3 outTexCoords[TEXTURE_TYPE_COUNT];

#include "utils.glsl"

void main() {		
    mat4 proj_view = globals.proj * globals.view;
    outPosition = (instance_matrix * vec4(vertex_position.xyz, 1.)).xyz;

    mat3 normal_matrix = mat3(transpose(inverse(instance_matrix)));
    outNormal = normal_matrix * vertex_normal;

    gl_Position = proj_view * vec4(outPosition.xyz, 1.);

    outMaterialIndex = instance_material_index;
    outColor = vertex_color;
    if (outMaterialIndex >= 0 && outMaterialIndex < uniforms.num_materials)
    {
        outColor *= uniforms.material_data[outMaterialIndex].base_color;
        for(int i = 0; i < TEXTURE_TYPE_COUNT; i++)
        {
            outTexIdx[i] = getTextureIndex(outMaterialIndex, i);
            outTexCoords[i] = getTextureCoords(outMaterialIndex, i);
        }
    }
}