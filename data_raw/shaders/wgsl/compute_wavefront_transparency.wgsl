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

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> scene_data: array<u32>;
@group(1) @binding(0) var<storage, read> gbuffer_material: array<f32>;
@group(1) @binding(1) var<storage, read_write> queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(2) var<storage, read_write> queue: array<TransparentHit>;

fn table_offset(kind: u32) -> u32 { return scene_data[4u + kind * 4u]; }
fn append_transparent(hit: TransparentHit) {
    let slot = atomicAdd(&queue_header[0].count, 1u);
    if (slot < queue_header[0].capacity) { queue[slot] = hit; }
    else { atomicStore(&queue_header[0].overflowed, 1u); }
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let width = u32(constant_data.screen_width * constant_data.render_scale);
    let height = u32(constant_data.screen_height * constant_data.render_scale);
    if (id.x >= width || id.y >= height) { return; }
    let pixel = id.y * width + id.x;
    let offset = pixel * SIZE_OF_DATA_BUFFER_ELEMENT;
    let material_id = u32(max(gbuffer_material[offset], 0.0));
    let flags = scene_data[table_offset(8u) + material_id * 48u + 47u];
    if ((flags & MATERIAL_FLAGS_ALPHAMODE_BLEND) == 0u) { return; }
    let base = table_offset(8u) + material_id * 48u + 20u;
    let color = vec4<f32>(bitcast<f32>(scene_data[base]), bitcast<f32>(scene_data[base + 1u]), bitcast<f32>(scene_data[base + 2u]), bitcast<f32>(scene_data[base + 3u]));
    append_transparent(TransparentHit(pixel, material_id, 0.0, clamp(color.a, 0.0, 1.0), color, 0u, 0u));
}
