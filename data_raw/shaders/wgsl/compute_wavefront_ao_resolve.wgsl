#import "common.inc"

struct AoState {
    visibility: f32,
    radius: f32,
    sample_count: u32,
    flags: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> gbuffer_material: array<f32>;
@group(0) @binding(2)
var<storage, read_write> ao_state: array<AoState>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let width = u32(constant_data.screen_width * constant_data.render_scale);
    let height = u32(constant_data.screen_height * constant_data.render_scale);
    if (id.x >= width || id.y >= height) { return; }
    let pixel = id.y * width + id.x;
    let offset = pixel * SIZE_OF_DATA_BUFFER_ELEMENT;
    ao_state[pixel] = AoState(clamp(gbuffer_material[offset + 2u], 0.0, 1.0), 1.0, 1u, 0u);
}
