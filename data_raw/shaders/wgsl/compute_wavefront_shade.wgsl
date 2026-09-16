#import "common.inc"
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

fn analytic_sky(direction: vec3<f32>) -> vec3<f32> {
    let height = clamp(direction.y * 0.5 + 0.5, 0.0, 1.0);
    return mix(vec3<f32>(0.015, 0.02, 0.04), vec3<f32>(0.12, 0.22, 0.45), height);
}

fn environment_radiance(direction: vec3<f32>) -> vec3<f32> {
    let fallback = analytic_sky(normalize(direction));
    if (constant_data.environment_texture_index == 0xffffffffu) {
        return fallback;
    }
    let d = normalize(direction);
    let uv = vec2<f32>(atan2(d.z, d.x) / (2.0 * MATH_PI) + 0.5, asin(clamp(d.y, -1.0, 1.0)) / MATH_PI + 0.5);
    let sampled = sample_texture(vec3<f32>(uv, f32(constant_data.environment_texture_index))).xyz;
    return select(fallback, max(sampled, vec3<f32>(0.0)), length(sampled) > MATH_EPSILON);
}

fn table_offset(kind: u32) -> u32 {
    return scene_data[4u + kind * 4u];
}

fn load_material(material_index: u32, word: u32) -> f32 {
    return bitcast<f32>(scene_data[table_offset(8u) + material_index * 48u + word]);
}

fn add_indirect(pixel_index: u32, value: vec3<f32>) {
    let current = indirect_accumulator[pixel_index].radiance_and_weight.xyz;
    indirect_accumulator[pixel_index].radiance_and_weight = vec4<f32>(current + value, 1.0);
}

@compute
@workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let hit_index = global_invocation_id.x;
    let hit_count = min(atomicLoad(&hit_queue_header[0].count), hit_queue_header[0].capacity);
    if (hit_index >= hit_count) {
        return;
    }
    let hit = hit_queue[hit_index];
    let ray = ray_queue[hit.ray_index];
    if (hit.visibility_id == 0u) {
        let environment = environment_radiance(ray.direction_and_t_max.xyz);
        let sky = environment * max(ray.throughput.xyz, vec3<f32>(0.0));
        add_indirect(ray.pixel_index, sky);
        return;
    }

    let instance_base = table_offset(3u) + hit.instance_id * 4u;
    let mesh_index = scene_data[instance_base + 1u];
    let mesh_base = table_offset(5u) + mesh_index * 16u;
    let material_index = max(bitcast<i32>(scene_data[mesh_base + 3u]), 0);
    let emissive = vec3<f32>(
        load_material(u32(material_index), 24u),
        load_material(u32(material_index), 25u),
        load_material(u32(material_index), 26u)
    ) * load_material(u32(material_index), 27u);
    let facing = max(dot(-ray.direction_and_t_max.xyz, normalize(vec3<f32>(0.0, 1.0, 0.0))), 0.0);
    let base_color = vec3<f32>(
        load_material(u32(material_index), 20u),
        load_material(u32(material_index), 21u),
        load_material(u32(material_index), 22u)
    );
    add_indirect(ray.pixel_index, ray.throughput.xyz * (emissive + base_color * facing * 0.05));
}
