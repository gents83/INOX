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

void main() {		
    gl_Position = (globals.proj * globals.view * instance_matrix * vec4(vertex_position.xyz, 1.));
    vec4 material_color = uniforms.material_data[instance_material_index].base_color;
    outColor = vertex_color * material_color;
}