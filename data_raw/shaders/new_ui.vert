#version 450
#extension GL_GOOGLE_include_directive : require

#include "common.glsl"

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec3 vertex_tangent;
layout(location = 3) in vec4 vertex_color;
layout(location = 4) in vec2 vertex_tex_coord[MAX_TEXTURE_COORDS_SETS];

layout(location = 14) in int  instance_material_index;


//Output
layout(location = 0) out vec4 outColor;
layout(location = 1) out uint  outTexIdx[TEXTURE_TYPE_COUNT];
layout(location = 9) out vec3 outTexCoords[TEXTURE_TYPE_COUNT];

#include "utils.glsl"

void main() {
    gl_Position =
        vec4(2.0 * vertex_position.x / globals.screen_size.x - 1.0,
              2.0 * vertex_position.y / globals.screen_size.y - 1.0, 
              vertex_position.z, 
              1.0);
    // egui encodes vertex colors in gamma spaces, so we must decode the colors here:
    outColor = vertex_color / 255.;  

    if (instance_material_index >= 0 && instance_material_index < uniforms.num_materials)
    {
        for(int i = 0; i < TEXTURE_TYPE_COUNT; i++)
        {
            outTexIdx[i] = getTextureIndex(instance_material_index, i);
            outTexCoords[i] = getTextureCoords(instance_material_index, i);
        }
    }
}