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
    let scale = clamp(constant_data.render_scale, 0.25, 1.0);
    let source_width = max(1u, u32(ceil(f32(width) * scale)));
    let source_height = max(1u, u32(ceil(f32(height) * scale)));
    let source_x = min(source_width - 1u, u32(f32(id.x) * scale));
    let source_y = min(source_height - 1u, u32(f32(id.y) * scale));
    let source_index = source_y * source_width + source_x;
    let destination_index = id.y * width + id.x;
    destination[destination_index] = source[source_index];
}
