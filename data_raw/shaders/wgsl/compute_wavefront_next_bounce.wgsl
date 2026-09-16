#import "common.inc"
#import "utils.inc"
#import "wavefront_records.inc"

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> scene_data: array<u32>;
@group(1) @binding(0) var<storage, read> gbuffer_world: array<f32>;
@group(1) @binding(1) var<storage, read> gbuffer_material: array<f32>;
@group(1) @binding(2) var<storage, read> ray_queue: array<WavefrontRay>;
@group(1) @binding(3) var<storage, read> hit_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(4) var<storage, read> hit_queue: array<WavefrontHit>;
@group(1) @binding(5) var<storage, read_write> next_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(6) var<storage, read_write> next_queue: array<WavefrontRay>;

fn append_next(ray: WavefrontRay) {
    let slot = atomicAdd(&next_queue_header[0].count, 1u);
    if (slot < next_queue_header[0].capacity) { next_queue[slot] = ray; }
    else { atomicStore(&next_queue_header[0].overflowed, 1u); }
}
fn hash_u32(v: u32) -> u32 { var x=v; x ^= x>>16u; x*=0x7feb352du; x ^= x>>15u; x*=0x846ca68bu; return x^(x>>16u); }
fn random01(seed: u32) -> f32 { return (f32(hash_u32(seed) & 0x00ffffffu) + 0.5) / 16777216.0; }
fn table_offset(kind: u32) -> u32 { return scene_data[4u + kind * 4u]; }
fn load_material(material_id: u32, word: u32) -> f32 { return bitcast<f32>(scene_data[table_offset(8u) + material_id * 48u + word]); }
fn safe_normalize(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let length_squared = dot(value, value);
    return select(fallback, value * inverseSqrt(max(length_squared, MATH_EPSILON)), length_squared > MATH_EPSILON);
}
fn fresnel_schlick(cosine: f32, ior: f32) -> f32 {
    let r0 = (1.0 - ior) / (1.0 + ior);
    let base = r0 * r0;
    return base + (1.0 - base) * pow(1.0 - clamp(cosine, 0.0, 1.0), 5.0);
}
fn cosine_sample(normal: vec3<f32>, seed: u32) -> vec3<f32> {
    let phi = 2.0 * MATH_PI * random01(seed);
    let radius = sqrt(random01(seed ^ 0x9e3779b9u));
    let tangent = safe_normalize(select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(normal.x) > 0.5) - normal * normal.y, vec3<f32>(1.0, 0.0, 0.0));
    let bitangent = safe_normalize(cross(normal, tangent), vec3<f32>(0.0, 0.0, 1.0));
    let local = vec3<f32>(radius * cos(phi), radius * sin(phi), sqrt(max(0.0, 1.0 - radius * radius)));
    return safe_normalize(tangent * local.x + bitangent * local.y + normal * local.z, normal);
}

@compute
@workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let count = min(atomicLoad(&hit_queue_header[0].count), hit_queue_header[0].capacity);
    if (index >= count) { return; }
    let hit = hit_queue[index];
    let ray = ray_queue[hit.ray_index];
    if (hit.visibility_id == 0u || ray.bounce >= constant_data.max_path_depth) { return; }
    let offset = ray.pixel_index * SIZE_OF_DATA_BUFFER_ELEMENT;
    let surface_normal = safe_normalize(octahedral_unmapping(unpack2x16float(bitcast<u32>(gbuffer_world[offset + 3u]))), vec3<f32>(0.0, 1.0, 0.0));
    let material_id = max(bitcast<i32>(bitcast<u32>(gbuffer_material[offset])), 0);
    let roughness = clamp(load_material(u32(material_id), 16u), 0.045, 1.0);
    let metallic = clamp(load_material(u32(material_id), 17u), 0.0, 1.0);
    let transmission = clamp(load_material(u32(material_id), 19u), 0.0, 1.0);
    let ior = clamp(load_material(u32(material_id), 18u), 1.001, 3.0);
    let base_color = clamp(vec3<f32>(load_material(u32(material_id), 20u), load_material(u32(material_id), 21u), load_material(u32(material_id), 22u)), vec3<f32>(0.0), vec3<f32>(1.0));
    let incoming = safe_normalize(ray.direction_and_t_max.xyz, vec3<f32>(0.0, -1.0, 0.0));
    let view = -incoming;
    let front_face = dot(view, surface_normal) >= 0.0;
    let normal = select(-surface_normal, surface_normal, front_face);
    let eta = select(ior, 1.0 / ior, front_face);
    let cosine = clamp(dot(view, normal), 0.0, 1.0);
    let seed = hash_u32(ray.pixel_index ^ constant_data.frame_index ^ ray.bounce * 31u ^ ray.sample_index);
    let reflection = safe_normalize(reflect(-view, normal), normal);
    let candidate_refraction = refract(-view, normal, eta);
    let tir = dot(candidate_refraction, candidate_refraction) <= MATH_EPSILON;
    let reflectance = fresnel_schlick(cosine, ior);
    let choose_reflection = metallic > 0.5 || roughness < 0.2 || transmission <= 0.0 || tir || random01(seed) < reflectance;
    var direction = reflection;
    var pdf = max(reflectance, 0.001);
    var bsdf = mix(base_color / MATH_PI, vec3<f32>(1.0), metallic);
    if (!choose_reflection) {
        direction = safe_normalize(candidate_refraction, reflection);
        pdf = max((1.0 - reflectance) * transmission, 0.001);
        bsdf = base_color * transmission;
    } else if (metallic < 0.5 && roughness >= 0.2 && transmission <= 0.0) {
        let glossy = cosine_sample(normal, seed ^ 0xa511e9b3u);
        direction = safe_normalize(mix(reflection, glossy, roughness * roughness), reflection);
        let cosine_out = max(dot(normal, direction), 0.0);
        pdf = max(cosine_out / MATH_PI, 0.001);
    }
    let cosine_out = max(dot(normal, direction), 0.0);
    var throughput = ray.throughput.xyz * bsdf * cosine_out / pdf;
    throughput = clamp(throughput, vec3<f32>(0.0), vec3<f32>(65504.0));
    if (ray.bounce >= constant_data.min_roulette_depth) {
        let survival = clamp(max(max(throughput.x, throughput.y), throughput.z), 0.05, 0.99);
        if (random01(seed ^ 0x68bc21ebu) > survival) { return; }
        throughput /= survival;
    }
    let origin_position = vec3<f32>(gbuffer_world[offset], gbuffer_world[offset + 1u], gbuffer_world[offset + 2u]);
    let origin = origin_position + normal * select(-HIT_EPSILON, HIT_EPSILON, dot(direction, normal) >= 0.0);
    append_next(WavefrontRay(vec4<f32>(origin, HIT_EPSILON), vec4<f32>(direction, MAX_TRACING_DISTANCE), vec4<f32>(throughput, 1.0), ray.pixel_index, ray.bounce + 1u, ray.flags, ray.sample_index));
}
