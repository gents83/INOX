#import "common.inc"
#import "utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var finalize_texture: texture_2d<f32>;
@group(0) @binding(2)
var visibility_texture: texture_2d<f32>;
@group(0) @binding(3)
var radiance_texture: texture_2d<f32>;
@group(0) @binding(4)
var depth_texture: texture_depth_2d;
@group(0) @binding(5)
var debug_data_texture: texture_2d<f32>;


fn debug_color_override(color: vec4<f32>, pixel: vec2<u32>) -> vec4<f32> {
    var out_color = color;
    if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS) != 0) {
        let visibility_output = textureLoad(visibility_texture, pixel, 0);
        let visibility_id = pack4x8unorm(visibility_output);
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            let meshlet_id = (visibility_id >> 8u); 
            let meshlet_color = hash(meshlet_id + 1u);
            out_color = vec4<f32>(vec3<f32>(
                f32(meshlet_color & 255u),
                f32((meshlet_color >> 8u) & 255u),
                f32((meshlet_color >> 16u) & 255u)
            ) / 255., 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_VISIBILITY_BUFFER) != 0) {
        out_color = textureLoad(visibility_texture, pixel, 0);
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER) != 0) {
        let depth = textureLoad(depth_texture, pixel, 0);
        let v = vec3<f32>(1. - depth);
        out_color = vec4<f32>(v, 1.);
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE) != 0) {
        let v = textureLoad(debug_data_texture, pixel, 0);
    } 
    return out_color;
}

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
    let d = vec2<f32>(textureDimensions(debug_data_texture));
    let pixel_coords = vec2<u32>(u32(v_in.uv.x * d.x), u32(v_in.uv.y * d.y));

    var out_color = vec4<f32>(0.);    
    out_color = debug_color_override(out_color, pixel_coords); 
    return out_color;
}