#version 450
#extension GL_GOOGLE_include_directive : require

#include "common.glsl"

layout(std430, binding = 2) buffer UIData
{
    float scale;
} 
ui_data;

//Vertex
layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec2 vertex_tex_coord;
layout(location = 2) in uint vertex_color;
//Instance
layout(location = 7) in vec4 instance_draw_area;
layout(location = 8) in mat4 instance_matrix;
layout(location = 12) in mat3 instance_normal_matrix;
layout(location = 15) in int instance_material_index;

layout(location = 0) out vec4 out_color;
layout(location = 1) out int out_material_index;
layout(location = 2) out vec3 out_tex_coord;


#include "utils.glsl"


void main() {
    float ui_scale = ui_data.scale;
    gl_Position =
        vec4( 2. * vertex_position.x * ui_scale / constant_data.screen_size.x - 1.,
              1. - 2. * vertex_position.y * ui_scale/ constant_data.screen_size.y, 
              vertex_position.z, 
              1.
            );
    uint support_srbg = constant_data.flags & CONSTANT_DATA_FLAGS_SUPPORT_SRGB;
    vec4 color = rgbaFromInteger(vertex_color);
    if (support_srbg == 0u) {
        out_color = vec4(color.rgba / 255.);
    } else {
        out_color = vec4(linearFromSrgb(color.rgb), color.a / 255.0);  
    }
    out_material_index = instance_material_index;

    if (instance_material_index >= 0)
    {
        out_tex_coord = getTextureCoords(instance_material_index, TEXTURE_TYPE_BASE_COLOR, vertex_tex_coord);
    }
}