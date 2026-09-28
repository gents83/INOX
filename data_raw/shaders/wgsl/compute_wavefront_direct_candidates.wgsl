#import "common.inc"
#import "packed_scene.inc"
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
var<storage, read> gbuffer_material: array<f32>;
@group(1) @binding(2)
var<storage, read_write> reservoirs: array<Reservoir>;
@group(1) @binding(3)
var<storage, read_write> ao_state: array<AoState>;
@group(1) @binding(4)
var<storage, read_write> direct_accumulator: array<Accumulator>;

fn hash_u32(v: u32) -> u32 {
    var x = v;
    x ^= x >> 16u;
    x *= 0x7feb352du;
    x ^= x >> 15u;
    x *= 0x846ca68bu;
    return x ^ (x >> 16u);
}
fn finite_scalar(value: f32) -> bool { return value == value && abs(value) < 65504.0; }
fn finite_vector(value: vec3<f32>) -> bool { return all(value == value) && all(abs(value) < vec3<f32>(65504.0)); }
const MAX_RESERVOIR_CANDIDATES: u32 = 65535u;
fn add_candidate_count(current: u32, added: u32) -> u32 {
    return min(MAX_RESERVOIR_CANDIDATES, current + min(added, MAX_RESERVOIR_CANDIDATES - min(current, MAX_RESERVOIR_CANDIDATES)));
}
fn valid_weight(value: f32) -> bool { return finite_scalar(value) && value > 0.000001; }
fn safe_weight(numerator: f32, denominator: f32) -> f32 {
    if (!finite_scalar(numerator) || !valid_weight(denominator)) { return 0.0; }
    let value = numerator / denominator;
    return select(0.0, value, valid_weight(value));
}
fn safe_normalize(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let length_squared = dot(value, value);
    return select(fallback, value * inverseSqrt(max(length_squared, MATH_EPSILON)),
        finite_scalar(length_squared) && length_squared > MATH_EPSILON);
}
fn light_alias_count() -> u32 {
    if (!packed_scene_valid()) { return 0u; }
    let words = table_words(11u);
    let offset = table_offset(11u);
    if (words < PACKED_LIGHT_ALIAS_HEADER_WORDS || !range_has_words(11u, 0u, words)
        || scene_data[offset] != PACKED_LIGHT_ALIAS_VERSION) { return 0u; }
    return min(scene_data[offset + 1u], (words - PACKED_LIGHT_ALIAS_HEADER_WORDS) / PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS);
}
fn sample_light_alias(seed: u32, count: u32) -> vec2<u32> {
    let words = table_words(11u);
    if (count == 0u || !packed_table_range_valid(11u, 0u, words, table_stride(11u))
        || count > (words - PACKED_LIGHT_ALIAS_HEADER_WORDS) / PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS) { return vec2<u32>(0u); }
    let table_relative = PACKED_LIGHT_ALIAS_HEADER_WORDS;
    let bucket = hash_u32(seed) % count;
    let record_relative = table_relative + bucket * PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS;
    if (!range_has_words(11u, record_relative, PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS)) { return vec2<u32>(0u); }
    let record = table_offset(11u) + record_relative;
    let q = load_scene_f32(record + 1u);
    let coin = f32(hash_u32(seed ^ 0x9e3779b9u)) / 4294967296.0;
    let alias_index = min(scene_data[record], count - 1u);
    let selected = min(select(bucket, alias_index, coin >= q), count - 1u);
    let selected_relative = table_relative + selected * PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS;
    if (!range_has_words(11u, selected_relative, PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS)) { return vec2<u32>(0u); }
    let selected_record = table_offset(11u) + selected_relative;
    return vec2<u32>(selected, bitcast<u32>(load_scene_f32(selected_record + 2u)));
}
fn evaluate_light(position: vec3<f32>, normal: vec3<f32>, light_id: u32) -> vec3<f32> {
    if (!scene_index_valid(10u, light_id, 16u)) { return vec3<f32>(0.0); }
    let light_base = table_offset(10u) + light_id * 16u;
    let light_position = vec3<f32>(load_scene_f32(light_base), load_scene_f32(light_base + 1u), load_scene_f32(light_base + 2u));
    let light_color = vec3<f32>(load_scene_f32(light_base + 8u), load_scene_f32(light_base + 9u), load_scene_f32(light_base + 10u));
    let intensity = load_scene_f32(light_base + 7u);
    let to_light = light_position - position;
    let distance_sq = dot(to_light, to_light);
    if (!finite_scalar(distance_sq) || distance_sq <= 0.0001 || !finite_scalar(intensity) || !finite_vector(light_color)) {
        return vec3<f32>(0.0);
    }
    let contribution = max(dot(normal, safe_normalize(to_light, vec3<f32>(0.0, 1.0, 0.0))), 0.0)
        * max(intensity, 0.0) / distance_sq * max(light_color, vec3<f32>(0.0));
    return select(vec3<f32>(0.0), contribution, finite_vector(contribution));
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!packed_scene_valid()) { return; }
    let width = u32(constant_data.screen_width);
    let height = u32(constant_data.screen_height);
    if (id.x >= width || id.y >= height) { return; }
    let pixel = id.y * width + id.x;
    let data_index = pixel * SIZE_OF_DATA_BUFFER_ELEMENT;
    if (data_index > arrayLength(&gbuffer_world) || arrayLength(&gbuffer_world) - data_index < SIZE_OF_DATA_BUFFER_ELEMENT
        || pixel >= arrayLength(&reservoirs) || pixel >= arrayLength(&ao_state)
        || pixel >= arrayLength(&direct_accumulator)) { return; }
    let scene_light_words = table_words(10u);
    let light_count = scene_light_words / 16u;
    let alias_count = light_alias_count();
    let candidates = select(min(light_count, 4u), min(alias_count, 4u), alias_count > 0u);
    var reservoir = Reservoir(0u, 0u, 0.0, 0.0, 0.0, 0u, 0u, 0.0);
    var selected = vec3<f32>(0.0);
    var selected_weight = 0.0;
    let normal = safe_normalize(octahedral_unmapping(unpack2x16float(u32(gbuffer_world[data_index + 3u]))), vec3<f32>(0.0, 1.0, 0.0));
    let position = vec3<f32>(gbuffer_world[data_index], gbuffer_world[data_index + 1u], gbuffer_world[data_index + 2u]);

    // Temporal reuse: retain the previous reservoir only when it is valid.
    let previous = reservoirs[pixel];
    if (light_count > 0u && previous.age > 0u && previous.flags == constant_data.history_epoch && previous.selected_light_id < light_count) {
        let contribution = evaluate_light(position, normal, previous.selected_light_id);
        let target_value = max(max(contribution.x, contribution.y), contribution.z) + 0.000001;
        let reuse_pdf = previous.target_pdf;
        let confidence = select(0.0, clamp(previous.confidence, 0.1, 10.0), finite_scalar(previous.confidence));
        let weight = safe_weight(target_value, reuse_pdf) * confidence;
        if (valid_weight(weight)) {
            reservoir = previous;
            reservoir.age = min(previous.age + 1u, 65535u);
            reservoir.weight_sum = weight;
            reservoir.selected_weight = weight;
            reservoir.target_pdf = reuse_pdf;
            reservoir.candidate_count = min(max(previous.candidate_count, 1u), MAX_RESERVOIR_CANDIDATES);
            selected = contribution;
            selected_weight = weight;
        }
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
        if (light_count > 0u && neighbor.age > 0u && neighbor.flags == constant_data.history_epoch && neighbor.selected_light_id < light_count) {
            let contribution = evaluate_light(position, normal, neighbor.selected_light_id);
            let target_value = max(max(contribution.x, contribution.y), contribution.z) + 0.000001;
            let reuse_pdf = neighbor.target_pdf;
            let base_weight = safe_weight(target_value, reuse_pdf);
            let jacobian = select(0.0, clamp(base_weight, 0.1, 10.0), valid_weight(base_weight));
            let confidence = select(0.0, clamp(neighbor.confidence, 0.1, 10.0), finite_scalar(neighbor.confidence));
            let weight = base_weight * jacobian * confidence;
            if (valid_weight(weight)) {
                reservoir.candidate_count = add_candidate_count(reservoir.candidate_count, max(min(neighbor.candidate_count, MAX_RESERVOIR_CANDIDATES), 1u));
                reservoir.weight_sum = select(reservoir.weight_sum, reservoir.weight_sum + weight, finite_scalar(reservoir.weight_sum + weight));
                if (valid_weight(reservoir.weight_sum) && hash_u32(pixel ^ constant_data.frame_index ^ neighbor_index * 0x9e3779b9u) % max(reservoir.candidate_count, 1u) == 0u) {
                    reservoir.selected_light_id = neighbor.selected_light_id;
                    reservoir.selected_weight = weight;
                    reservoir.target_pdf = reuse_pdf;
                    selected = contribution;
                    selected_weight = weight;
                }
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
        let weight = safe_weight(target_value, proposal_pdf);
        if (valid_weight(weight)) {
            let next_weight_sum = reservoir.weight_sum + weight;
            if (finite_scalar(next_weight_sum)) {
                reservoir.candidate_count = add_candidate_count(reservoir.candidate_count, 1u);
                reservoir.weight_sum = next_weight_sum;
                if (hash_u32(pixel ^ candidate ^ constant_data.frame_index) % max(reservoir.candidate_count, 1u) == 0u) {
                    reservoir.selected_light_id = light_id;
                    reservoir.target_pdf = proposal_pdf;
                    reservoir.selected_weight = weight;
                    selected = contribution;
                    selected_weight = weight;
                }
            }
        }
    }
    reservoir.candidate_count = min(reservoir.candidate_count, MAX_RESERVOIR_CANDIDATES);
    if (reservoir.candidate_count > 0u) { reservoir.age = max(reservoir.age, 1u); }
    let valid_reservoir = reservoir.candidate_count > 0u
        && valid_weight(reservoir.weight_sum)
        && valid_weight(reservoir.target_pdf)
        && valid_weight(selected_weight)
        && finite_vector(selected);
    let denominator = f32(max(reservoir.candidate_count, 1u)) * selected_weight;
    var estimate = vec3<f32>(0.0);
    if (valid_reservoir && valid_weight(denominator)) {
        let candidate_estimate = selected * reservoir.weight_sum / denominator;
        if (finite_vector(candidate_estimate)) { estimate = candidate_estimate; }
    }
    // Direct illumination must be modulated by the hit material. Without this lookup,
    // Cornell's red/green walls collapse to the same neutral light response.
    var base_color = vec3<f32>(1.0);
    var primary_emission = vec3<f32>(0.0);
    let material_value = gbuffer_material[data_index];
    if (finite_scalar(material_value) && material_value >= 0.0 && material_value < f32(MAX_NUM_MATERIALS)) {
        let material_id = u32(material_value);
        if (scene_index_valid(8u, material_id, 48u)) {
            let material_color = vec3<f32>(
                load_scene_f32(table_offset(8u) + material_id * 48u + 20u),
                load_scene_f32(table_offset(8u) + material_id * 48u + 21u),
                load_scene_f32(table_offset(8u) + material_id * 48u + 22u)
            );
            if (finite_vector(material_color)) { base_color = max(material_color, vec3<f32>(0.0)); }
            let material_emission = vec3<f32>(
                load_scene_f32(table_offset(8u) + material_id * 48u + 24u),
                load_scene_f32(table_offset(8u) + material_id * 48u + 25u),
                load_scene_f32(table_offset(8u) + material_id * 48u + 26u)
            );
            let emission_strength = load_scene_f32(table_offset(8u) + material_id * 48u + 27u);
            if (finite_vector(material_emission) && finite_scalar(emission_strength)) {
                primary_emission = max(material_emission, vec3<f32>(0.0)) * max(emission_strength, 0.0);
            }
        }
    }
    // A visible emissive surface contributes even when there are no punctual
    // lights or the stochastic light reservoir is empty.
    estimate = estimate * base_color + primary_emission;
    reservoir.confidence = 0.0;
    if (valid_reservoir) {
        let candidate_confidence = selected_weight / reservoir.weight_sum;
        if (valid_weight(candidate_confidence)) { reservoir.confidence = candidate_confidence; }
    }
    reservoir.weight_sum = select(0.0, reservoir.weight_sum, valid_weight(reservoir.weight_sum));
    reservoir.target_pdf = select(0.0, reservoir.target_pdf, valid_weight(reservoir.target_pdf));
    reservoir.selected_weight = select(0.0, reservoir.selected_weight, valid_weight(reservoir.selected_weight));
    reservoir.flags = constant_data.history_epoch;
    reservoirs[pixel] = reservoir;
    ao_state[pixel] = AoState(1.0, 1.0, 1u, 0u);
    direct_accumulator[pixel] = Accumulator(vec4<f32>(estimate, 1.0), vec2<f32>(0.0), 0u, 0u);
}
