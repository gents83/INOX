#import "common.inc"
#import "utils.inc"
#import "wavefront_records.inc"

struct Reservoir {
    selected_light_id: u32,
    candidate_count: u32,
    weight_sum: f32,
    target_pdf: f32,
    selected_weight: f32,
    age: u32,
    flags: u32,
    confidence: f32,
};

struct AoState {
    visibility: f32,
    radius: f32,
    sample_count: u32,
    flags: u32,
};

struct Accumulator {
    radiance_and_weight: vec4<f32>,
    first_and_second_moment: vec2<f32>,
    history_length: u32,
    flags: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> scene_data: array<u32>;
@group(1) @binding(0)
var<storage, read> gbuffer_world: array<f32>;
@group(1) @binding(1)
var<storage, read_write> reservoirs: array<Reservoir>;
@group(1) @binding(2)
var<storage, read_write> ao_state: array<AoState>;
@group(1) @binding(3)
var<storage, read_write> direct_accumulator: array<Accumulator>;

fn table_offset(kind: u32) -> u32 {
    return scene_data[4u + kind * 4u];
}
fn table_words(kind: u32) -> u32 {
    return scene_data[5u + kind * 4u];
}
fn hash_u32(v: u32) -> u32 {
    var x = v;
    x ^= x >> 16u;
    x *= 0x7feb352du;
    x ^= x >> 15u;
    x *= 0x846ca68bu;
    return x ^ (x >> 16u);
}
fn load_scene_f32(offset: u32) -> f32 { return bitcast<f32>(scene_data[offset]); }
fn light_alias_count() -> u32 {
    let words = table_words(11u);
    let offset = table_offset(11u);
    if (words < PACKED_LIGHT_ALIAS_HEADER_WORDS || scene_data[offset] != PACKED_LIGHT_ALIAS_VERSION) { return 0u; }
    return min(scene_data[offset + 1u], (words - PACKED_LIGHT_ALIAS_HEADER_WORDS) / PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS);
}
fn sample_light_alias(seed: u32, count: u32) -> vec2<u32> {
    let table = table_offset(11u) + PACKED_LIGHT_ALIAS_HEADER_WORDS;
    let bucket = hash_u32(seed) % count;
    let record = table + bucket * PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS;
    let q = load_scene_f32(record + 1u);
    let coin = f32(hash_u32(seed ^ 0x9e3779b9u)) / 4294967296.0;
    let alias_index = min(scene_data[record], count - 1u);
    let selected = min(select(bucket, alias_index, coin >= q), count - 1u);
    let selected_record = table + selected * PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS;
    return vec2<u32>(selected, bitcast<u32>(load_scene_f32(selected_record + 2u)));
}
fn evaluate_light(position: vec3<f32>, normal: vec3<f32>, light_id: u32) -> vec3<f32> {
    let light_base = table_offset(10u) + light_id * 16u;
    let light_position = vec3<f32>(load_scene_f32(light_base), load_scene_f32(light_base + 1u), load_scene_f32(light_base + 2u));
    let light_color = vec3<f32>(load_scene_f32(light_base + 8u), load_scene_f32(light_base + 9u), load_scene_f32(light_base + 10u));
    let intensity = load_scene_f32(light_base + 7u);
    let to_light = light_position - position;
    let distance_sq = max(dot(to_light, to_light), 0.0001);
    return max(dot(normal, normalize(to_light)), 0.0) * intensity / distance_sq * max(light_color, vec3<f32>(0.0));
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let width = u32(constant_data.screen_width);
    let height = u32(constant_data.screen_height);
    if (id.x >= width || id.y >= height) { return; }
    let pixel = id.y * width + id.x;
    let data_index = pixel * SIZE_OF_DATA_BUFFER_ELEMENT;
    let scene_light_words = table_words(10u);
    let light_count = scene_light_words / 16u;
    let alias_count = light_alias_count();
    let candidates = select(min(light_count, 4u), min(alias_count, 4u), alias_count > 0u);
    var reservoir = Reservoir(0u, 0u, 0.0, 0.0, 0.0, 0u, 0u, 0.0);
    var selected = vec3<f32>(0.0);
    var selected_weight = 0.0;
    let normal = octahedral_unmapping(unpack2x16float(u32(gbuffer_world[data_index + 3u])));
    let position = vec3<f32>(gbuffer_world[data_index], gbuffer_world[data_index + 1u], gbuffer_world[data_index + 2u]);

    // Temporal reuse: retain the previous reservoir only when it is valid.
    let previous = reservoirs[pixel];
    if (light_count > 0u && previous.age > 0u && previous.selected_light_id < light_count) {
        let contribution = evaluate_light(position, normal, previous.selected_light_id);
        let target_value = max(max(contribution.x, contribution.y), contribution.z) + 0.000001;
        let reuse_pdf = max(previous.target_pdf, 0.000001);
        let weight = target_value / reuse_pdf * clamp(previous.confidence, 0.1, 10.0);
        reservoir = previous;
        reservoir.age = min(previous.age + 1u, 65535u);
        reservoir.weight_sum = weight;
        reservoir.selected_weight = weight;
        reservoir.target_pdf = reuse_pdf;
        reservoir.candidate_count = max(previous.candidate_count, 1u);
        selected = contribution;
        selected_weight = weight;
    }

    // Reuse a bounded four-neighbor cross after temporal validity checks.
    var offsets: array<vec2<i32>, 4>;
    offsets[0] = vec2<i32>(-1, 0);
    offsets[1] = vec2<i32>(1, 0);
    offsets[2] = vec2<i32>(0, -1);
    offsets[3] = vec2<i32>(0, 1);
    for (var neighbor_index = 0u; neighbor_index < 4u; neighbor_index += 1u) {
        let neighbor_coord = vec2<i32>(i32(id.x), i32(id.y)) + offsets[neighbor_index];
        if (neighbor_coord.x < 0 || neighbor_coord.y < 0 || neighbor_coord.x >= i32(width) || neighbor_coord.y >= i32(height)) { continue; }
        let neighbor_pixel = u32(neighbor_coord.y) * width + u32(neighbor_coord.x);
        let neighbor = reservoirs[neighbor_pixel];
        if (light_count > 0u && neighbor.age > 0u && neighbor.selected_light_id < light_count) {
            let contribution = evaluate_light(position, normal, neighbor.selected_light_id);
            let target_value = max(max(contribution.x, contribution.y), contribution.z) + 0.000001;
            let reuse_pdf = max(neighbor.target_pdf, 0.000001);
            let jacobian = clamp(target_value / reuse_pdf, 0.1, 10.0);
            let weight = target_value / reuse_pdf * jacobian * clamp(neighbor.confidence, 0.1, 10.0);
            reservoir.candidate_count += max(neighbor.candidate_count, 1u);
            reservoir.weight_sum += weight;
            if (hash_u32(pixel ^ constant_data.frame_index ^ neighbor_index * 0x9e3779b9u) % max(reservoir.candidate_count, 1u) == 0u) {
                reservoir.selected_light_id = neighbor.selected_light_id;
                reservoir.selected_weight = weight;
                reservoir.target_pdf = reuse_pdf;
                selected = contribution;
                selected_weight = weight;
            }
        }
    }

    for (var candidate = 0u; candidate < candidates; candidate += 1u) {
        let seed = pixel ^ (constant_data.frame_index + candidate * 17u);
        var light_id = 0u;
        var proposal_pdf = 0.0;
        if (alias_count > 0u) {
            let sample = sample_light_alias(seed, alias_count);
            light_id = sample.x;
            proposal_pdf = max(bitcast<f32>(sample.y), 0.000001);
        } else if (light_count > 0u) {
            light_id = hash_u32(seed) % light_count;
            proposal_pdf = 1.0 / f32(light_count);
        }
        if (proposal_pdf <= 0.0 || light_id >= light_count) { continue; }
        let contribution = evaluate_light(position, normal, light_id);
        let target_value = max(max(contribution.x, contribution.y), contribution.z) + 0.000001;
        let weight = target_value / proposal_pdf;
        reservoir.candidate_count += 1u;
        reservoir.weight_sum += weight;
        if (hash_u32(pixel ^ candidate ^ constant_data.frame_index) % reservoir.candidate_count == 0u) {
            reservoir.selected_light_id = light_id;
            reservoir.target_pdf = proposal_pdf;
            reservoir.selected_weight = weight;
            selected = contribution;
            selected_weight = weight;
        }
    }
    if (reservoir.candidate_count > 0u) { reservoir.age = max(reservoir.age, 1u); }
    reservoir.confidence = select(0.0, selected_weight / reservoir.weight_sum, reservoir.weight_sum > 0.0);
    reservoirs[pixel] = reservoir;
    ao_state[pixel] = AoState(1.0, 1.0, 1u, 0u);
    direct_accumulator[pixel] = Accumulator(vec4<f32>(selected, 1.0), vec2<f32>(0.0), 0u, 0u);
}
