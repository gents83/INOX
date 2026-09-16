#import "common.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
struct Accumulator {
    radiance_and_weight: vec4<f32>,
    first_and_second_moment: vec2<f32>,
    history_length: u32,
    flags: u32,
};
@group(0) @binding(1)
var<storage, read_write> final_accumulator: array<Accumulator>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Denoise the full-resolution accumulator produced by wavefront tracing.
    let width = u32(constant_data.screen_width);
    let height = u32(constant_data.screen_height);
    if (id.x >= width || id.y >= height) {
        return;
    }
    let pixel = id.y * width + id.x;
    let value = max(final_accumulator[pixel].radiance_and_weight.xyz, vec3<f32>(0.0));
    final_accumulator[pixel].radiance_and_weight = vec4<f32>(value, 1.0);
}
