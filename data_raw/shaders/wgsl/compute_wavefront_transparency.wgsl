#import "common.inc"
#import "packed_scene.inc"
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

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> scene_data: array<u32>;
@group(1) @binding(0) var<storage, read> gbuffer_material: array<f32>;
@group(1) @binding(1) var<storage, read_write> queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(2) var<storage, read_write> queue: array<TransparentHit>;

fn append_transparent(hit: TransparentHit) {
    let slot = atomicAdd(&queue_header[0].count, 1u);
    if (slot < queue_header[0].capacity) { queue[slot] = hit; }
    else {
        atomicStore(&queue_header[0].overflowed, 1u);
        atomicAdd(&queue_header[0].dropped_records, 1u);
    }
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!packed_scene_valid()) { return; }
    // G-buffer and transparency queues are full-resolution resources. Render
    // scale is applied only by the reconstruction/upscale tail.
    let width = u32(constant_data.screen_width);
    let height = u32(constant_data.screen_height);
    if (id.x >= width || id.y >= height) { return; }
    let pixel = id.y * width + id.x;
    let offset = pixel * SIZE_OF_DATA_BUFFER_ELEMENT;
    if (offset > arrayLength(&gbuffer_material) || arrayLength(&gbuffer_material) - offset < SIZE_OF_DATA_BUFFER_ELEMENT) { return; }
    let material_value = gbuffer_material[offset];
    if (material_value != material_value || material_value < 0.0 || material_value >= f32(MAX_NUM_MATERIALS)) {
        return;
    }
    let material_id = u32(material_value);
    if (!scene_index_valid(8u, material_id, 48u)) { return; }
    let material_base = table_offset(8u) + material_id * 48u;
    let flags = scene_data[material_base + 47u];
    if ((flags & MATERIAL_FLAGS_ALPHAMODE_BLEND) == 0u) { return; }
    let base = material_base + 20u;
    let color = vec4<f32>(bitcast<f32>(scene_data[base]), bitcast<f32>(scene_data[base + 1u]), bitcast<f32>(scene_data[base + 2u]), bitcast<f32>(scene_data[base + 3u]));
    append_transparent(TransparentHit(pixel, material_id, 0.0, clamp(color.a, 0.0, 1.0), color, 0u, 0u));
}
