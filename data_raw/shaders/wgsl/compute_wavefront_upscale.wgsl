#import "common.inc"

struct Accumulator {
    radiance_and_weight: vec4<f32>,
    first_and_second_moment: vec2<f32>,
    history_length: u32,
    flags: u32,
};

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> source: array<Accumulator>;
@group(0) @binding(2) var<storage, read_write> destination: array<Accumulator>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let width = u32(constant_data.screen_width);
    let height = u32(constant_data.screen_height);
    if (id.x >= width || id.y >= height) { return; }
    // Source and destination are both full-resolution accumulators.  Keep
    // render_scale as a reconstruction-quality control rather than changing
    // the storage stride (future filters may use it for kernel selection).
    let source_index = id.y * width + id.x;
    let destination_index = source_index;
    destination[destination_index] = source[source_index];
}
