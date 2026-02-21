struct BRDFSample {
    direction: vec3<f32>,
    pdf: f32,
    weight: vec3<f32>,
    is_specular: bool,
}

struct RayPackedData {
    origin_tmin: vec4<f32>,
    direction_tmax: vec4<f32>,
}

struct RadiancePackedData {
    data: vec4<f32>,
}

struct ThroughputPackedData {
    data: vec4<f32>,
}

fn pixarOnb(n: vec3f) -> mat3x3f {
    // https://www.jcgt.org/published/0006/01/01/paper-lowres.pdf
    let s = select(-1f, 1f, n.z >= 0f);
    let a = -1f / (s + n.z);
    let b = n.x * n.y * a;
    let u = vec3(1f + s * n.x * n.x * a, s * b, -s * n.x);
    let v = vec3(b, s + n.y * n.y * a, -n.y);

    return mat3x3(u, v, n);
}

fn sample_sphere(random_numbers: vec2<f32>) -> vec3<f32> {
    let z = 2. * random_numbers.y - 1.;
    let phi = 2. * MATH_PI * random_numbers.x;
    let s = sqrt(1. - z * z);
    let x = cos(phi) * s;
    let y = sin(phi) * s;
    return normalize(vec3<f32>(x, y, z));
}

fn sample_hemisphere(random_numbers: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
    let v = sample_sphere(random_numbers);
    return v * sign(dot(v, n));
}

fn sample_cosine_wheighted_hemisphere(random_numbers: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
    let phi = 2. * MATH_PI * random_numbers.y;
    let sin_theta = sqrt(1. - random_numbers.x);

    let x = cos(phi) * sin_theta;
    let y = sin(phi) * sin_theta;
    let z = sqrt(random_numbers.x);

    let v = vec3<f32>(x, y, z);
    let onb = pixarOnb(n);
    return normalize(onb * v);
}

fn sample_in_cone(random_numbers: vec2<f32>, cos_max: f32, n: vec3<f32>) -> vec3<f32> {
    let cos_theta = 1. - random_numbers.x * (1. - cos_max);
    let sin_theta = sqrt(1. - cos_theta * cos_theta);
    let phi = 2. * MATH_PI * random_numbers.y;

    let x = cos(phi) * sin_theta;
    let y = sin(phi) * sin_theta;
    let z = cos_theta;
    
    let v = vec3<f32>(x, y, z);
    let onb = pixarOnb(n);
    return normalize(onb * v);
}

// GGX importance sampling - sample microfacet normal from GGX distribution
fn sample_ggx(random_numbers: vec2<f32>, roughness: f32, n: vec3<f32>) -> vec3<f32> {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;

    let phi = 2. * MATH_PI * random_numbers.x;
    let cos_theta = sqrt((1. - random_numbers.y) / (1. + (alpha2 - 1.) * random_numbers.y));
    let sin_theta = sqrt(1. - cos_theta * cos_theta);

    // Half vector in local space
    let h_local = vec3<f32>(
        cos(phi) * sin_theta,
        sin(phi) * sin_theta,
        cos_theta
    );

    let onb = pixarOnb(n);
    return normalize(onb * h_local);
}

// GGX PDF for the sampled direction
fn ggx_pdf(NdotH: f32, VdotH: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let d = (NdotH * NdotH) * (alpha2 - 1.) + 1.;
    let D = alpha2 / (MATH_PI * d * d);
    return D * NdotH / (4. * VdotH + MATH_EPSILON);
}

// Cosine-weighted hemisphere PDF
fn cosine_pdf(NdotL: f32) -> f32 {
    return NdotL / MATH_PI;
}


// Sample BRDF: choose between diffuse or specular based on material properties
fn sample_brdf(
    normal: vec3<f32>,
    view: vec3<f32>,
    roughness: f32,
    metallic: f32,
    seed: ptr<function, vec2<u32>>
) -> BRDFSample {
    let random = get_random_numbers(seed);
    let random2 = get_random_numbers(seed);

    // Probability of choosing specular vs diffuse sampling
    // Higher metallic = more specular, lower roughness = more specular
    let specular_probability = mix(0.04, 1.0, metallic);

    var result: BRDFSample;

    if (random.x < specular_probability) {
        // Specular sampling: GGX importance sampling
        let perceptual_roughness = max(roughness, 0.045);
        let h = sample_ggx(random2, perceptual_roughness, normal);
        let l = reflect(-view, h);

        let NdotL = dot(normal, l);
        if (NdotL <= 0.) {
            result.direction = l;
            result.pdf = 0.;
            result.weight = vec3<f32>(0.);
            result.is_specular = true;
            return result;
        }

        let NdotH = max(dot(normal, h), 0.);
        let VdotH = max(dot(view, h), 0.);

        result.direction = l;
        result.pdf = ggx_pdf(NdotH, VdotH, perceptual_roughness) * specular_probability;
        result.weight = vec3<f32>(1. / specular_probability);
        result.is_specular = true;
    } else {
        // Diffuse sampling: cosine-weighted hemisphere
        result.direction = sample_cosine_wheighted_hemisphere(random2, normal);
        let NdotL = max(dot(normal, result.direction), 0.);
        result.pdf = cosine_pdf(NdotL) * (1. - specular_probability);
        result.weight = vec3<f32>(1. / (1. - specular_probability));
        result.is_specular = false;
    }

    return result;
}

// Note: trace_shadow_ray and trace_ao_ray are defined directly in shaders
// that include raytracing.inc (which provides traverse_bvh).

// Russian roulette path termination - returns true if path should be terminated
fn russian_roulette(throughput: vec3<f32>, seed: ptr<function, vec2<u32>>) -> bool {
    let max_component = max(max(throughput.x, throughput.y), throughput.z);
    let survival_probability = min(max_component, 0.95);
    let random = get_random_numbers(seed);
    return random.x >= survival_probability;
}

// Russian roulette throughput adjustment (divide by survival probability)
fn russian_roulette_weight(throughput: vec3<f32>) -> vec3<f32> {
    let max_component = max(max(throughput.x, throughput.y), throughput.z);
    let survival_probability = min(max_component, 0.95);
    return throughput / survival_probability;
}

// Pack G-buffer data into the 3 data buffers
fn write_gbuffer(
    data_index: u32,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    material_id: u32,
    tangent: vec4<f32>,
    uv0: vec2<f32>,
    instance_id: u32
) {
    // data_buffer_0: world_pos.xyz (origin), material_id (t_min)
    data_buffer_0[data_index].origin_tmin = vec4<f32>(world_pos, f32(material_id));
    // data_buffer_0: normal (direction), tangent.w (t_max)
    data_buffer_0[data_index].direction_tmax = vec4<f32>(normal, tangent.w);

    // data_buffer_1: initialized to 0 (Radiance)
    data_buffer_1[data_index].data = vec4<f32>(0.);

    // data_buffer_2: packed_tangent, instance_id, packed_uv0, (reserved for shadow_mask)
    data_buffer_2[data_index].data = vec4<f32>(
        f32(pack2x16float(octahedral_mapping(tangent.xyz))),
        f32(instance_id),
        f32(pack2x16float(uv0)),
        0.
    );
}

// Read G-buffer world position from data_buffer_0
fn read_gbuffer_world_pos(data_index: u32) -> vec3<f32> {
    return data_buffer_0[data_index].origin_tmin.xyz;
}

// Read G-buffer normal from data_buffer_0
fn read_gbuffer_normal(data_index: u32) -> vec3<f32> {
    return data_buffer_0[data_index].direction_tmax.xyz;
}

// Read G-buffer material ID from data_buffer_0
fn read_gbuffer_material_id(data_index: u32) -> u32 {
    return u32(data_buffer_0[data_index].origin_tmin.w);
}

// Read G-buffer tangent from data_buffer_2[0] + data_buffer_0
fn read_gbuffer_tangent(data_index: u32) -> vec4<f32> {
    let packed_tangent = octahedral_unmapping(unpack2x16float(u32(data_buffer_2[data_index].data.x)));
    let tangent_w = data_buffer_0[data_index].direction_tmax.w;
    return vec4<f32>(packed_tangent, tangent_w);
}

// Read G-buffer UV0 from data_buffer_2[2] (packed as 2xf16)
fn read_gbuffer_uv0(data_index: u32) -> vec2<f32> {
    return unpack2x16float(u32(data_buffer_2[data_index].data.z));
}

// Read G-buffer instance ID from data_buffer_2
fn read_gbuffer_instance_id(data_index: u32) -> u32 {
    return u32(data_buffer_2[data_index].data.y);
}

// Write shadow mask into data_buffer_2.w
fn write_shadow_mask(data_index: u32, shadow_mask: u32) {
    data_buffer_2[data_index].data.w = bitcast<f32>(shadow_mask);
}

// Read shadow mask from data_buffer_2.w
fn read_shadow_mask(data_index: u32) -> u32 {
    return bitcast<u32>(data_buffer_2[data_index].data.w);
}

// Write AO factor into data_buffer_1[2]
fn write_ao_factor(data_index: u32, ao: f32) {
    data_buffer_1[data_index].data.z = ao;
}

// Read AO factor from data_buffer_1[2]
fn read_ao_factor(data_index: u32) -> f32 {
    return data_buffer_1[data_index].data.z;
}

// Write ray data for indirect bounces
fn write_ray_data(data_index: u32, origin: vec3<f32>, direction: vec3<f32>) {
    data_buffer_0[data_index].origin_tmin = vec4<f32>(origin, HIT_EPSILON);
    data_buffer_0[data_index].direction_tmax = vec4<f32>(direction, MAX_TRACING_DISTANCE);
}

// Read ray origin from data_buffer_0
fn read_ray_origin(data_index: u32) -> vec3<f32> {
    return data_buffer_0[data_index].origin_tmin.xyz;
}

// Read ray t_min from data_buffer_0
fn read_ray_t_min(data_index: u32) -> f32 {
    return data_buffer_0[data_index].origin_tmin.w;
}

// Read ray direction from data_buffer_0
fn read_ray_direction(data_index: u32) -> vec3<f32> {
    return data_buffer_0[data_index].direction_tmax.xyz;
}

// Read ray t_max from data_buffer_0
fn read_ray_t_max(data_index: u32) -> f32 {
    return data_buffer_0[data_index].direction_tmax.w;
}

// Write hit data (repurposes db0)
fn write_hit_data(data_index: u32, hit_pos: vec3<f32>, visibility_id: u32) {
    data_buffer_0[data_index].origin_tmin = vec4<f32>(hit_pos, 0.);
    data_buffer_0[data_index].direction_tmax = vec4<f32>(0., 0., 0., f32(visibility_id));
}

// Read hit position from db0
fn read_hit_position(data_index: u32) -> vec3<f32> {
    return data_buffer_0[data_index].origin_tmin.xyz;
}

// Read hit visibility ID from db0
fn read_hit_visibility_id(data_index: u32) -> u32 {
    return u32(data_buffer_0[data_index].direction_tmax.w);
}

// Save incoming ray direction to db1.w
fn save_incoming_direction(data_index: u32, direction: vec3<f32>) {
    data_buffer_1[data_index].data.w = f32(pack2x16float(octahedral_mapping(direction)));
}

// Read incoming ray direction from db1.w
fn read_incoming_direction(data_index: u32) -> vec3<f32> {
    return octahedral_unmapping(unpack2x16float(u32(data_buffer_1[data_index].data.w)));
}

// Write radiance data
fn write_radiance(data_index: u32, radiance: vec3<f32>, visibility_id: u32) {
    // Note: visibility_id here might be unused or stored in w?
    // IndirectSetup wrote visibility_id to w?
    // But save_incoming_direction writes to w.
    // If we write radiance, we should preserve w if it matters.
    // But Radiance is written by Shade pass, which generates Next Ray.
    // Next Ray (IndirectBounce) starts with Traversal which reads Next Ray from db0.
    // So db1 is just accumulation.
    // save_incoming_direction is done by Traversal.
    // So between Traversal and Shade, db1.w holds Direction.
    // Shade reads Direction, computes Lighting, adds to db1.xyz.
    // Does Shade need to preserve db1.w?
    // No, once read, it's done.
    // So write_radiance can overwrite w?
    // Wait, write_radiance is used by Setup and Diffuse.
    // They write 0 or material ID to w.
    // Traversal overwrites w with Direction.
    // Shade reads Direction.
    // Shade accumulates to xyz. It should preserve w? Or doesn't matter?
    // If Shade uses += for xyz, it needs to access components.
    // data_buffer_1[index].data.x += ...

    // This helper writes vec4.
    data_buffer_1[data_index].data = vec4<f32>(radiance, f32(visibility_id));
}

// Read radiance
fn read_radiance(data_index: u32) -> vec3<f32> {
    return data_buffer_1[data_index].data.xyz;
}

// Write throughput data
fn write_throughput(data_index: u32, throughput: vec3<f32>, is_active: bool) {
    data_buffer_2[data_index].data = vec4<f32>(throughput, select(0., 1., is_active));
}

// Read throughput
fn read_throughput(data_index: u32) -> vec3<f32> {
    return data_buffer_2[data_index].data.xyz;
}

// Read active flag
fn read_ray_active(data_index: u32) -> bool {
    return data_buffer_2[data_index].data.w > 0.5;
}
