#import "common.inc"

struct Accumulator {
    radiance_and_weight: vec4<f32>,
    first_and_second_moment: vec2<f32>,
    history_length: u32,
    flags: u32,
};
struct AoState {
    visibility: f32,
    radius: f32,
    sample_count: u32,
    flags: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read_write> final_accumulator: array<Accumulator>;
@group(0) @binding(2)
var<storage, read> direct_accumulator: array<Accumulator>;
@group(0) @binding(3)
var<storage, read> ao_state: array<AoState>;
@group(0) @binding(4)
var<storage, read> indirect_accumulator: array<Accumulator>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let width = u32(constant_data.screen_width * constant_data.render_scale);
    let height = u32(constant_data.screen_height * constant_data.render_scale);
    if (id.x >= width || id.y >= height) { return; }
    let pixel = id.y * width + id.x;
    let direct = direct_accumulator[pixel].radiance_and_weight.xyz;
    let indirect = indirect_accumulator[pixel].radiance_and_weight.xyz;
    let ao = clamp(ao_state[pixel].visibility, 0.0, 1.0);
    let result = direct * ao + indirect;
    let luminance = dot(result, vec3<f32>(0.2126, 0.7152, 0.0722));
    final_accumulator[pixel].radiance_and_weight = vec4<f32>(result, 1.0);
    final_accumulator[pixel].first_and_second_moment = vec2<f32>(luminance, luminance * luminance);
    final_accumulator[pixel].history_length = min(final_accumulator[pixel].history_length + 1u, 65535u);
}
