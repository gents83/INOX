#import "utils.wgsl"
#import "common.wgsl"


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
@group(1) @binding(0)
var<storage, read> textures: Textures;

#import "texture_utils.wgsl"


@vertex
fn vs_main(
    v_in: UIVertex,
    i_in: UIInstance,
) -> VertexOutput {

    let ui_scale = 2.;

    var vertex_out: VertexOutput;
    vertex_out.clip_position = vec4<f32>(
        2. * v_in.position.x * ui_scale / constant_data.screen_width - 1.,
        1. - 2. * v_in.position.y * ui_scale / constant_data.screen_height,
        0.001 * f32(i_in.index),
        1.
    );
    let color = u32(v_in.color);
    let c = unpack_color(color);
    //vertex_out.color = vec4<f32>(linear_from_srgb(c.rgb), c.a / 255.0);
    vertex_out.color = vec4<f32>(c / 255.);
    vertex_out.tex_coords = vec3<f32>(v_in.uv.xy, f32(i_in.texture_index));

    return vertex_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = sample_texture(v_in.tex_coords);
    let final_color = vec4<f32>(v_in.color .rgb * texture_color.rgb, v_in.color.a);
    return final_color;
}