#import "common.inc"
#import "packed_scene.inc"
#import "utils.inc"
#import "wavefront_records.inc"
#import "texture_utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> scene_data: array<u32>;

@group(1) @binding(0)
var<storage, read> ray_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(1)
var<storage, read> ray_queue: array<WavefrontRay>;
@group(1) @binding(2)
var<storage, read> hit_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(3)
var<storage, read> hit_queue: array<WavefrontHit>;
struct Accumulator {
    radiance_and_weight: vec4<f32>,
    first_and_second_moment: vec2<f32>,
    history_length: u32,
    flags: u32,
};

@group(1) @binding(4)
var<storage, read_write> indirect_accumulator: array<Accumulator>;
@group(1) @binding(5)
var<uniform> textures: Textures;

fn finite_scalar(value: f32) -> bool {
    return value == value && abs(value) < 65504.0;
}
fn finite_vector(value: vec3<f32>) -> bool {
    return all(value == value) && all(abs(value) < vec3<f32>(65504.0));
}
fn sanitize_vector(value: vec3<f32>) -> vec3<f32> {
    return select(vec3<f32>(0.0), clamp(value, vec3<f32>(0.0), vec3<f32>(65504.0)), finite_vector(value));
}
fn safe_normalize(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let length_squared = dot(value, value);
    return select(fallback, value * inverseSqrt(max(length_squared, MATH_EPSILON)),
        finite_scalar(length_squared) && length_squared > MATH_EPSILON);
}

fn analytic_sky(direction: vec3<f32>) -> vec3<f32> {
    let height = clamp(direction.y * 0.5 + 0.5, 0.0, 1.0);
    return mix(vec3<f32>(0.015, 0.02, 0.04), vec3<f32>(0.12, 0.22, 0.45), height);
}

fn environment_radiance(direction: vec3<f32>) -> vec3<f32> {
    let fallback = analytic_sky(safe_normalize(direction, vec3<f32>(0.0, 1.0, 0.0)));
    let texture_index = constant_data.environment_texture_index;
    if (texture_index >= MAX_NUM_TEXTURES) {
        return fallback;
    }
    let d = safe_normalize(direction, vec3<f32>(0.0, 1.0, 0.0));
    let uv = vec2<f32>(atan2(d.z, d.x) / (2.0 * MATH_PI) + 0.5, asin(clamp(d.y, -1.0, 1.0)) / MATH_PI + 0.5);
    let sampled = sample_texture(vec3<f32>(uv, f32(texture_index))).xyz;
    return sanitize_vector(sampled);
}

fn load_material(material_index: u32, word: u32) -> f32 {
    if (!scene_index_valid(8u, material_index, 48u) || word >= 48u) { return 0.0; }
    return bitcast<f32>(scene_data[table_offset(8u) + material_index * 48u + word]);
}

fn add_indirect(pixel_index: u32, value: vec3<f32>) {
    if (pixel_index >= arrayLength(&indirect_accumulator) || !finite_vector(value)) { return; }
    let current = sanitize_vector(indirect_accumulator[pixel_index].radiance_and_weight.xyz);
    indirect_accumulator[pixel_index].radiance_and_weight = vec4<f32>(sanitize_vector(current + value), 1.0);
}

@compute
@workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    if (!packed_scene_valid()) { return; }
    let hit_index = global_invocation_id.x;
    let hit_count = min(atomicLoad(&hit_queue_header[0].count), hit_queue_header[0].capacity);
    if (hit_index >= hit_count) {
        return;
    }
    let hit = hit_queue[hit_index];
    if (hit.ray_index >= arrayLength(&ray_queue)) { return; }
    let ray = ray_queue[hit.ray_index];
    if (!finite_vector(ray.throughput.xyz) || !finite_vector(ray.direction_and_t_max.xyz)) { return; }
    if (hit.visibility_id == 0u) {
        let environment = environment_radiance(ray.direction_and_t_max.xyz);
        let sky = environment * max(ray.throughput.xyz, vec3<f32>(0.0));
        add_indirect(ray.pixel_index, sky);
        return;
    }

    if (!scene_index_valid(3u, hit.instance_id, 4u)) { return; }
    let instance_base = table_offset(3u) + hit.instance_id * 4u;
    let mesh_index = scene_data[instance_base + 1u];
    if (!scene_index_valid(5u, mesh_index, 16u)) { return; }
    let mesh_base = table_offset(5u) + mesh_index * 16u;
    let material_index = bitcast<i32>(scene_data[mesh_base + 3u]);
    if (material_index == INVALID_INDEX || material_index < 0 || material_index >= i32(MAX_NUM_MATERIALS)
        || !scene_index_valid(8u, u32(material_index), 48u)) {
        return;
    }
    let material_id = u32(material_index);
    let emissive = sanitize_vector(vec3<f32>(
        load_material(material_id, 24u),
        load_material(material_id, 25u),
        load_material(material_id, 26u)
    ) * max(load_material(material_id, 27u), 0.0));
    let base_color = sanitize_vector(vec3<f32>(
        load_material(material_id, 20u),
        load_material(material_id, 21u),
        load_material(material_id, 22u)
    ));
    // The shade queue does not carry a geometric normal. Treat emissive
    // materials as explicitly two-sided rather than inventing a world-up
    // facing term, which biases emission based on camera direction.
    add_indirect(ray.pixel_index, sanitize_vector(ray.throughput.xyz * (emissive + base_color * 0.05)));
}
