#import "common.wgsl"

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) tex_coords_0: vec2<f32>,
    @location(5) tex_coords_1: vec2<f32>,
    @location(6) tex_coords_2: vec2<f32>,
    @location(7) tex_coords_3: vec2<f32>,
};

struct InstanceInput {
    @builtin(instance_index) index: u32,
    @location(8) draw_area: vec4<f32>,
    @location(9) model_matrix_0: vec4<f32>,
    @location(10) model_matrix_1: vec4<f32>,
    @location(11) model_matrix_2: vec4<f32>,
    @location(12) model_matrix_3: vec4<f32>,
    @location(13) material_index: i32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) world_tangent: vec4<f32>,
    @location(4) view: vec3<f32>,
    @location(5) @interpolate(flat) material_index: i32,
    @location(6) tex_coords_base_color: vec2<f32>,
    @location(7) tex_coords_metallic_roughness: vec2<f32>,
    @location(8) tex_coords_normal: vec2<f32>,
    @location(9) tex_coords_emissive: vec2<f32>,
    @location(10) tex_coords_occlusion: vec2<f32>,
    @location(11) tex_coords_specular_glossiness: vec2<f32>,
    @location(12) tex_coords_diffuse: vec2<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;