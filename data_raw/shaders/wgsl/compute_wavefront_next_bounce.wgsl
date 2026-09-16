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
fn finite_scalar(value: f32) -> bool { return value == value && abs(value) < 65504.0; }
fn finite_vector(value: vec3<f32>) -> bool { return all(value == value) && all(abs(value) < vec3<f32>(65504.0)); }
fn finite_or(value: f32, fallback: f32) -> f32 { return select(fallback, value, finite_scalar(value)); }
fn safe_normalize(value: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let length_squared = dot(value, value);
    return select(fallback, value * inverseSqrt(max(length_squared, MATH_EPSILON)), finite_scalar(length_squared) && length_squared > MATH_EPSILON);
}
fn fresnel_schlick(cosine: f32, ior: f32) -> f32 {
    let r0 = (1.0 - ior) / (1.0 + ior);
    let base = r0 * r0;
    return clamp(base + (1.0 - base) * pow(1.0 - clamp(cosine, 0.0, 1.0), 5.0), 0.0, 1.0);
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
    let roughness = clamp(finite_or(load_material(u32(material_id), 16u), 1.0), 0.045, 1.0);
    let metallic = clamp(finite_or(load_material(u32(material_id), 17u), 0.0), 0.0, 1.0);
    let transmission = clamp(finite_or(load_material(u32(material_id), 19u), 0.0), 0.0, 1.0) * (1.0 - metallic);
    let ior = clamp(finite_or(load_material(u32(material_id), 18u), 1.5), 1.001, 3.0);
    let base_color = clamp(vec3<f32>(
        finite_or(load_material(u32(material_id), 20u), 0.8),
        finite_or(load_material(u32(material_id), 21u), 0.8),
        finite_or(load_material(u32(material_id), 22u), 0.8)), vec3<f32>(0.0), vec3<f32>(1.0));
    let incoming = safe_normalize(ray.direction_and_t_max.xyz, vec3<f32>(0.0, -1.0, 0.0));
    let view = -incoming;
    let front_face = dot(view, surface_normal) >= 0.0;
    let normal = select(-surface_normal, surface_normal, front_face);
    let eta = select(ior, 1.0 / ior, front_face);
    let cosine = clamp(dot(view, normal), 0.0, 1.0);
    let seed = hash_u32(ray.pixel_index ^ constant_data.frame_index ^ ray.bounce * 31u ^ ray.sample_index);
    let reflection = safe_normalize(reflect(-view, normal), normal);
    let candidate_refraction = refract(-view, normal, eta);
    let tir = !finite_vector(candidate_refraction) || dot(candidate_refraction, candidate_refraction) <= MATH_EPSILON;
    let reflectance = fresnel_schlick(cosine, ior);
    let specular_probability = clamp(mix(reflectance, 1.0, metallic), 0.0, 1.0);
    let transmission_probability = clamp(transmission * (1.0 - specular_probability), 0.0, 1.0);
    let diffuse_probability = max(1.0 - specular_probability - transmission_probability, 0.0);
    let probability_sum = max(specular_probability + transmission_probability + diffuse_probability, MATH_EPSILON);
    let p_specular = specular_probability / probability_sum;
    let p_transmission = transmission_probability / probability_sum;
    let selector = random01(seed);

    var direction = reflection;
    var bsdf = vec3<f32>(1.0);
    var p_lobe = 1.0;
    var p_direction = 1.0;
    var throughput: vec3<f32>;
    if (tir) {
        // Total internal reflection collapses every transmission choice into a
        // single delta reflection event. Its lobe and direction probabilities
        // are therefore both one, independent of the original transmission mix.
        direction = reflection;
        bsdf = vec3<f32>(1.0);
        p_lobe = 1.0;
        p_direction = 1.0;
        throughput = ray.throughput.xyz;
        if (!finite_vector(throughput)) { return; }
    } else if (selector < p_transmission && transmission_probability > 0.0) {
        direction = safe_normalize(candidate_refraction, reflection);
        bsdf = base_color;
        p_lobe = p_transmission;
    } else if (selector < p_transmission + p_specular && p_specular > 0.0) {
        p_lobe = p_specular;
        if (roughness >= 0.2 && metallic < 0.999) {
            direction = cosine_sample(reflection, seed ^ 0xa511e9b3u);
            if (dot(normal, direction) <= 0.0) {
                direction = reflection;
                p_direction = 1.0;
            } else {
                p_direction = max(dot(reflection, direction), 0.0) / MATH_PI;
            }
            bsdf = vec3<f32>(1.0);
        }
    } else if (diffuse_probability > 0.0) {
        p_lobe = max(1.0 - p_specular - p_transmission, MATH_EPSILON);
        direction = cosine_sample(normal, seed ^ 0x51ed270bu);
        p_direction = max(dot(normal, direction), 0.0) / MATH_PI;
        bsdf = base_color / MATH_PI;
    }

    if (!finite_vector(direction) || !finite_vector(bsdf) || !finite_scalar(p_lobe) || !finite_scalar(p_direction)
        || p_lobe <= MATH_EPSILON || p_direction <= MATH_EPSILON) { return; }
    if (!tir) {
        let cosine_out = abs(dot(normal, direction));
        if (!finite_scalar(cosine_out) || cosine_out <= MATH_EPSILON) { return; }
        throughput = ray.throughput.xyz * bsdf * cosine_out / (p_lobe * p_direction);
        if (!finite_vector(throughput)) { return; }
    }
    throughput = clamp(throughput, vec3<f32>(0.0), vec3<f32>(65504.0));
    if (length(throughput) <= MATH_EPSILON) { return; }
    if (ray.bounce >= constant_data.min_roulette_depth) {
        let survival = clamp(max(max(throughput.x, throughput.y), throughput.z), 0.05, 0.99);
        if (!finite_scalar(survival)) { return; }
        if (random01(seed ^ 0x68bc21ebu) > survival) { return; }
        throughput /= survival;
        if (!finite_vector(throughput)) { return; }
    }
    let origin_position = vec3<f32>(gbuffer_world[offset], gbuffer_world[offset + 1u], gbuffer_world[offset + 2u]);
    let origin = origin_position + normal * select(-HIT_EPSILON, HIT_EPSILON, dot(direction, normal) >= 0.0);
    if (!finite_vector(origin)) { return; }
    append_next(WavefrontRay(vec4<f32>(origin, HIT_EPSILON), vec4<f32>(direction, MAX_TRACING_DISTANCE), vec4<f32>(throughput, 1.0), ray.pixel_index, ray.bounce + 1u, ray.flags, ray.sample_index));
}
