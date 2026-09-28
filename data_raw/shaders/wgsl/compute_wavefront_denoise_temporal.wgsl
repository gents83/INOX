#import "common.inc"

struct Accumulator {
    radiance_and_weight: vec4<f32>,
    first_and_second_moment: vec2<f32>,
    history_length: u32,
    flags: u32,
};
@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read_write> final_accumulator: array<Accumulator>;
@group(0) @binding(2)
var<storage, read_write> history_accumulator: array<Accumulator>;

fn finite_scalar(value: f32) -> bool {
    return value == value && abs(value) < 65504.0;
}
fn sanitize(value: vec3<f32>) -> vec3<f32> {
    let valid = all(value == value) && all(abs(value) < vec3<f32>(65504.0));
    return select(vec3<f32>(0.0), clamp(value, vec3<f32>(0.0), vec3<f32>(65504.0)), valid);
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Denoise the full-resolution accumulator produced by wavefront tracing.
    let width = u32(constant_data.screen_width);
    let height = u32(constant_data.screen_height);
    if (id.x >= width || id.y >= height) { return; }
    let pixel = id.y * width + id.x;
    let current = final_accumulator[pixel];
    let previous = history_accumulator[pixel];
    let current_radiance = sanitize(current.radiance_and_weight.xyz);
    let previous_variance = max(previous.first_and_second_moment.y
        - previous.first_and_second_moment.x * previous.first_and_second_moment.x, 0.0);
    let history_valid = previous.flags == constant_data.history_epoch
        && previous.history_length > 0u
        && finite_scalar(previous.radiance_and_weight.x)
        && finite_scalar(previous.radiance_and_weight.y)
        && finite_scalar(previous.radiance_and_weight.z)
        && finite_scalar(previous.first_and_second_moment.x)
        && finite_scalar(previous.first_and_second_moment.y)
        && finite_scalar(previous_variance);
    let history_count = select(0u, min(previous.history_length, 65535u), history_valid);
    let blend_denominator = f32(history_count + 1u);
    let previous_radiance = sanitize(previous.radiance_and_weight.xyz);
    let blended = sanitize((current_radiance + previous_radiance * f32(history_count)) / blend_denominator);
    let luminance = dot(blended, vec3<f32>(0.2126, 0.7152, 0.0722));
    let second_moment = luminance * luminance;
    let next_history = Accumulator(
        vec4<f32>(blended, 1.0),
        vec2<f32>(luminance, second_moment),
        min(history_count + 1u, 65535u),
        constant_data.history_epoch,
    );
    final_accumulator[pixel] = next_history;
    history_accumulator[pixel] = next_history;
}
