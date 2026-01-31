#import "common.inc"
#import "pathtracing_common.inc"

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> surface_data: array<SurfaceData>;

@group(1) @binding(0) var<uniform> materials: Materials;
@group(1) @binding(1) var<uniform> textures: Textures;
@group(1) @binding(2) var<uniform> lights: Lights;
@group(1) @binding(3) var<storage, read_write> shadow_rays: array<ShadowRay>;
@group(1) @binding(4) var<storage, read_write> counters: PathTracingCounters;
@group(1) @binding(5) var<storage, read> rays: array<Ray>;
@group(1) @binding(6) var<storage, read_write> next_rays: array<Ray>;
@group(1) @binding(7) var<storage, read_write> data_buffer_1: array<f32>;

#import "utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "texture_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "light_utils.inc"

const WORKGROUP_SIZE: u32 = 8u;

struct LightSample {
    direction: vec3<f32>,
    distance: f32,
    radiance: vec3<f32>,
    pdf: f32,
}

fn pixarOnb(n: vec3f) -> mat3x3f {
    let s = select(-1f, 1f, n.z >= 0f);
    let a = -1f / (s + n.z);
    let b = n.x * n.y * a;
    let u = vec3(1f + s * n.x * n.x * a, s * b, -s * n.x);
    let v = vec3(b, s + n.y * n.y * a, -n.y);
    return mat3x3(u, v, n);
}

fn sample_cosine_weighted_hemisphere(random_numbers: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
    let phi = 2. * MATH_PI * random_numbers.y;
    let sin_theta = sqrt(1. - random_numbers.x);
    let x = cos(phi) * sin_theta;
    let y = sin(phi) * sin_theta;
    let z = sqrt(random_numbers.x);
    let v = vec3<f32>(x, y, z);

    let onb = pixarOnb(n);
    return normalize(onb * v);
}

fn sample_light_nee(light: ptr<function, LightData>, world_pos: vec3<f32>) -> LightSample {
    var L = vec3<f32>(0.);
    var dist = 10000.0;
    var radiance = vec3<f32>(0.);

    if ((*light).light_type == LIGHT_TYPE_DIRECTIONAL) {
        L = -normalize((*light).direction);
        dist = 10000.0;
        radiance = (*light).color * (*light).intensity;
    } else {
        let to_light = (*light).position - world_pos;
        dist = length(to_light);
        L = normalize(to_light);
        radiance = get_light_intensity(light, to_light);
    }
    return LightSample(L, dist, radiance, 1.0);
}

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let index = global_invocation_id.y * u32(constant_data.screen_width) + global_invocation_id.x;

    let ray = rays[index];
    if (ray.t_max <= 0.0) {
        return;
    }

    let s = surface_data[index];
    if (s.material_index < 0) {
        return;
    }

    // Unpack SurfaceData to PixelData
    var pixel_data: PixelData;
    pixel_data.world_pos = s.position;
    pixel_data.material_id = u32(s.material_index);
    pixel_data.color = vec4<f32>(s.albedo, 1.);
    pixel_data.normal = s.normal;
    pixel_data.tangent = s.tangent;
    pixel_data.uv_set[0] = vec4<f32>(s.uv, 0., 0.);
    pixel_data.uv_set[1] = vec4<f32>(0.);
    pixel_data.uv_set[2] = vec4<f32>(0.);
    pixel_data.uv_set[3] = vec4<f32>(0.);

    pixel_data.instance_id = 0u;
    // pixel_data.tangent = ??? We didn't store tangent.
    // Standard PBR might need tangent for normal mapping.
    // If we want normal mapping, we should store tangent frame in SurfaceData.
    // For now, we stored normal. If normal mapping was applied in Geometry pass, then `s.normal` is already perturbed normal?
    // In geometry pass, we compute:
    // normal = rotate_vector(normal, orientation);
    // But we don't apply normal map there.
    // Normal map is applied in `compute_color_from_material` -> `material_normal`.
    // It requires tangent.
    // I should add tangent to SurfaceData if I want normal maps.
    // The user requirement says "Reuse existing structures".
    // I will assume tangent is needed. But `SurfaceData` is limited size.
    // I'll skip tangent for now and assume geometry normal is enough or modify Geometry Pass later.

    let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);

    // Emissive
    if (length(material_info.f_emissive) > 0.0) {
        let emissive_radiance = ray.throughput * material_info.f_emissive;
        let data_index = ray.pixel_index * 4u;
        let r = data_buffer_1[data_index];
        let g = data_buffer_1[data_index + 1u];
        let b = data_buffer_1[data_index + 2u];
        data_buffer_1[data_index] = r + emissive_radiance.x;
        data_buffer_1[data_index + 1u] = g + emissive_radiance.y;
        data_buffer_1[data_index + 2u] = b + emissive_radiance.z;
    }

    let V = -ray.direction;
    let N = pixel_data.normal; // This might be perturbed by normal map inside compute_color if we had tangent.
    let hit_point = s.position;

    var seed = vec2<u32>(ray.pixel_index, constant_data.frame_index);
    let random = get_random_numbers(&seed);

    // NEE
    var shadow_ray: ShadowRay;
    shadow_ray.t_max = -1.0;
    if (constant_data.num_lights > 0u) {
        let light_idx = u32(random.x * f32(constant_data.num_lights));
        var light = lights.data[light_idx];
        let light_sample = sample_light_nee(&light, hit_point);
        let L = light_sample.direction;
        let NdotL = max(dot(N, L), 0.0);

        if (NdotL > 0.0 && light_sample.radiance.x > 0.) {
            shadow_ray.origin = hit_point + N * 0.001;
            shadow_ray.t_max = light_sample.distance - 0.002;
            shadow_ray.direction = L;
            shadow_ray.pixel_index = ray.pixel_index;
            let brdf = material_info.base_color.rgb / MATH_PI;
            shadow_ray.contribution = ray.throughput * light_sample.radiance * brdf * NdotL * f32(constant_data.num_lights);
            atomicAdd(&counters.shadow_ray_count, 1u);
        }
    }
    shadow_rays[ray.pixel_index] = shadow_ray;

    // Indirect
    var next_ray: Ray;
    next_ray.t_max = -1.0;
    if (ray.depth < constant_data.indirect_light_num_bounces) {
        let next_dir = sample_cosine_weighted_hemisphere(random, N);
        let NdotL_next = max(dot(N, next_dir), 0.0);
        if (NdotL_next > 0.0) {
            next_ray.origin = hit_point + N * 0.001;
            next_ray.direction = next_dir;
            next_ray.t_min = 0.0;
            next_ray.t_max = 10000.0;
            next_ray.pixel_index = ray.pixel_index;
            next_ray.depth = ray.depth + 1u;
            next_ray.throughput = ray.throughput * material_info.base_color.rgb;

            let p = max(next_ray.throughput.x, max(next_ray.throughput.y, next_ray.throughput.z));
            if (random.y < p) {
                next_ray.throughput = next_ray.throughput / p;
                atomicAdd(&counters.extension_ray_count, 1u);
            } else {
                next_ray.t_max = -1.0;
            }
        }
    }
    next_rays[ray.pixel_index] = next_ray;
}
