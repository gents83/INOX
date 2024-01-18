#import "common.inc"
#import "utils.inc"
#import "color_utils.inc"

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
var<storage, read_write> data_buffer_1: array<f32>;
@group(0) @binding(2)
var previous_texture: texture_2d<f32>;

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
    let dimensions = vec2<u32>(u32(constant_data.screen_width), u32(constant_data.screen_height));
    let pixel = vec2<f32>(v_in.uv.x * constant_data.screen_width, v_in.uv.y * constant_data.screen_height);

    let data_dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let data_scale = vec2<f32>(data_dimensions) / vec2<f32>(dimensions);
    let data_pixel = vec2<u32>(pixel * data_scale);
    let data_index = (data_pixel.y * data_dimensions.x + data_pixel.x) * SIZE_OF_DATA_BUFFER_ELEMENT;
    
    var radiance = vec3<f32>(0.);
    radiance.x = data_buffer_1[data_index];
    radiance.y = data_buffer_1[data_index + 1u];
    radiance.z = data_buffer_1[data_index + 2u];
    if(constant_data.frame_index > 0u) {
        let prev_value = textureLoad(previous_texture, data_pixel, 0).rgb;
        let frame_index = f32(min(constant_data.frame_index + 1u, 1000u));
        let weight = 1. / frame_index;
        radiance = mix(prev_value, radiance, weight);
    }   
     
    var out_color = vec4<f32>(radiance, 1.);   
    //out_color = vec4<f32>(tonemap_ACES_Hill(out_color.rgb), 1.);
    //out_color = vec4<f32>(linearTosRGB(out_color.rgb), 1.); 

    return out_color;
}
