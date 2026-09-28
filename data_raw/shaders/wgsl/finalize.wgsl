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
struct Accumulator {
    radiance_and_weight: vec4<f32>,
    first_and_second_moment: vec2<f32>,
    history_length: u32,
    flags: u32,
};

@group(0) @binding(1)
var<storage, read> final_accumulator: array<Accumulator>;
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

    // The accumulator follows the active render resolution. Do not use the
    // compile-time default dimensions when the window or render scale changes.
    let data_dimensions = dimensions;
    let data_pixel = min(
        vec2<u32>(pixel),
        data_dimensions - vec2<u32>(1u),
    );
    let pixel_index = data_pixel.y * data_dimensions.x + data_pixel.x;

    var radiance = final_accumulator[pixel_index].radiance_and_weight.xyz;
    if (constant_data.frame_index > 0u
        && final_accumulator[pixel_index].history_length > 0u
        && final_accumulator[pixel_index].flags == constant_data.history_epoch) {
        let prev_value = textureLoad(previous_texture, data_pixel, 0).rgb;
        let moments = final_accumulator[pixel_index].first_and_second_moment;
        let variance = max(moments.y - moments.x * moments.x, 0.0001);
        let sigma = sqrt(variance) * 2.0;
        let lower = max(radiance - vec3<f32>(sigma), vec3<f32>(0.0));
        let upper = radiance + vec3<f32>(sigma);
        let clamped_previous = clamp(prev_value, lower, upper);
        let frame_index = f32(min(final_accumulator[pixel_index].history_length, 1000u));
        let weight = 1.0 / max(frame_index, 1.0);
        radiance = mix(clamped_previous, radiance, weight);
    }
     
    var out_color = vec4<f32>(radiance, 1.);   
    return out_color;
}
