#import "common.inc"
#import "wavefront_records.inc"

struct TransparentHit {
    pixel_index: u32,
    material_id: u32,
    depth: f32,
    alpha: f32,
    color: vec4<f32>,
    layer: u32,
    flags: u32,
};
struct Accumulator {
    radiance_and_weight: vec4<f32>,
    first_and_second_moment: vec2<f32>,
    history_length: u32,
    flags: u32,
};

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> queue_header: array<WavefrontQueueHeader>;
@group(0) @binding(2) var<storage, read> queue: array<TransparentHit>;
@group(0) @binding(3) var<storage, read_write> final_accumulator: array<Accumulator>;

@compute
@workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let count = min(atomicLoad(&queue_header[0].count), queue_header[0].capacity);
    if (index >= count) { return; }
    let hit = queue[index];
    let current = final_accumulator[hit.pixel_index].radiance_and_weight.xyz;
    let alpha = clamp(hit.alpha, 0.0, 1.0);
    final_accumulator[hit.pixel_index].radiance_and_weight = vec4<f32>(hit.color.rgb * alpha + current * (1.0 - alpha), 1.0);
}
