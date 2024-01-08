#import "common.inc"
#import "utils.inc"

struct UIPassData {
    ui_scale: f32,
}

struct UIVertex {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: u32,
};

struct UIInstance {
    @builtin(instance_index) index: u32,
    @location(3) index_start: u32,
    @location(4) index_count: u32,
    @location(5) vertex_start: u32,
    @location(6) texture_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec3<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> ui_pass_data: UIPassData;
@group(1) @binding(0)
var<storage, read> textures: Textures;

#import "texture_utils.inc"
#import "color_utils.inc"


@vertex
fn vs_main(
    v_in: UIVertex,
    i_in: UIInstance,
) -> VertexOutput {

    let ui_scale = ui_pass_data.ui_scale;

    var vertex_out: VertexOutput;
    vertex_out.clip_position = vec4<f32>(
        2. * v_in.position.x * ui_scale / constant_data.screen_width - 1.,
        1. - 2. * v_in.position.y * ui_scale / constant_data.screen_height,
        0.001 * f32(i_in.index),
        1.
    );    
    vertex_out.color = unpack_color(u32(v_in.color));
    vertex_out.tex_coords = vec3<f32>(v_in.uv.xy, f32(i_in.texture_index));

    return vertex_out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_linear = sample_texture(in.tex_coords);

    //let tex_gamma = gamma_from_linear_rgba(tex_linear);
    //let out_color_gamma = in.color * tex_gamma;
    //return vec4<f32>(linear_from_gamma_rgb(out_color_gamma.rgb), out_color_gamma.a);

    let tex_gamma = gamma_from_linear_rgba(tex_linear);
    let out_color_gamma = in.color * tex_gamma;
    return out_color_gamma;
}