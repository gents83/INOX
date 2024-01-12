struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var source_texture: texture_2d<f32>;

#import "postfx_utils.inc"

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    //only one triangle, exceeding the viewport size
    let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
    let pos = vec4<f32>(uv * vec2<f32>(2., -2.) + vec2<f32>(-1., 1.), 0., 1.);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = pos;
    vertex_out.uv = uv;
    return vertex_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let d = vec2<f32>(textureDimensions(source_texture));
    let pixel_coords = vec2<f32>(f32(v_in.uv.x * d.x + 0.5), f32(v_in.uv.y * d.y + 0.5));

    var out_color = vec4<f32>(0.); 

    //out_color = textureLoad(source_texture, vec2<u32>(pixel_coords), 0); 
    out_color = vec4<f32>(fxaa(source_texture, pixel_coords, d), 1.);

    return out_color;
}