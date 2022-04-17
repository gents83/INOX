#version 450
#extension GL_GOOGLE_include_directive : require

#include "common.glsl"

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec3 vertex_tangent;
layout(location = 3) in vec4 vertex_color;
layout(location = 4) in vec2 vertex_tex_coord[MAX_TEXTURE_COORDS_SETS];
layout(location = 8) in uint instance_id;
layout(location = 9) in vec4 instance_draw_area;
layout(location = 10) in mat4 instance_matrix;
layout(location = 14) in mat3 instance_normal_matrix;
layout(location = 17) in int instance_material_index;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec3 out_position;
layout(location = 2) out vec3 out_normal;
layout(location = 3) out vec3 out_tex_coord[TEXTURE_TYPE_COUNT];
layout(location = 11) out int out_material_index;


#include "utils.glsl"


void main() {
    mat4 proj_view = constant_data.proj * constant_data.view;
    gl_Position = proj_view *(instance_matrix * vec4(vertex_position.xyz, 1.));
    
    out_normal = instance_normal_matrix * vertex_normal;    
    out_color = vertex_color;
    out_material_index = instance_material_index;

    if (instance_material_index >= 0)
    {
        for(uint i = TEXTURE_TYPE_BASE_COLOR; i < TEXTURE_TYPE_COUNT; i++)
        {
            uint tex_coords_set = getTextureCoordsSet(instance_material_index, i);
            out_tex_coord[i] = getTextureCoords(instance_material_index, i, vertex_tex_coord[tex_coords_set]);
        }
    }
}