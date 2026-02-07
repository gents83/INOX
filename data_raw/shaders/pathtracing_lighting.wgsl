#include "raytracing_bindings.inc"

@group(0) @binding(3) var<storage, read> surface_data: array<SurfaceData>;

// Group 1: Bindings unrelated to Scene/Geometry (Textures, Counters, Rays)
@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var<storage, read_write> shadow_rays: array<ShadowRay>;
@group(1) @binding(2) var<storage, read_write> counters: PathTracingCounters;
@group(1) @binding(3) var<storage, read> rays: array<Ray>;
@group(1) @binding(4) var<storage, read_write> next_rays: array<Ray>;
@group(1) @binding(5) var<storage, read_write> accumulator: array<RadiancePackedData>;
@group(1) @binding(6) var sampler_state: sampler;

const PI: f32 = 3.14159265359;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let ray_count = atomicLoad(&counters.extension_ray_count);

    if (index >= ray_count) {
        return;
    }

    let ray = rays[index];
    let surface = surface_data[ray.pixel_index];

    if (surface.flags == 0u) {
        return;
    }

    let material = get_material(u32(surface.material_index));
    let base_color = material.base_color.rgb;
    let roughness = material.roughness_factor;
    let metallic = material.metallic_factor;
    let emission = material.emissive_color * material.emissive_strength;

    // Add Emission
    // accumulator[ray.pixel_index] += emission * throughput?
    // Using simple accumulation for now
    // Atomic add for float? No.
    // Accumulator is ReadWrite.
    // If multiple rays hit same pixel? No, 1 ray per pixel per pass in this architecture.
    // accumulator is mapped by pixel_index.
    let current_acc = accumulator[ray.pixel_index].data;
    accumulator[ray.pixel_index].data = current_acc + vec4<f32>(emission * ray.throughput, 1.0);

    // NEE
    let num_lights = constant_data.num_lights;
    if (num_lights > 0u) {
        // Randomly select one light? Or iterate all?
        // For efficiency, pick one.
        // Seed?
        let light_idx = 0u; // Simplify
        let light = get_light(light_idx);

        let L = normalize(light.position - surface.position);
        let dist = length(light.position - surface.position);
        let N = normalize(surface.normal); // Use normal map later

        let NdotL = max(dot(N, L), 0.0);

        if (NdotL > 0.0) {
            var shadow_ray: ShadowRay;
            shadow_ray.origin = surface.position + N * 0.001;
            shadow_ray.direction = L;
            shadow_ray.t_max = dist - 0.002;
            shadow_ray.pixel_index = ray.pixel_index;

            // BRDF (Lambert + GGX)
            // Simplified Lambert
            let diffuse = base_color / PI;
            let radiance = light.color * light.intensity / (dist * dist);

            shadow_ray.contribution = radiance * diffuse * NdotL * ray.throughput;

            shadow_rays[ray.pixel_index] = shadow_ray;
        }
    }

    // Next Ray
    if (ray.depth < constant_data.num_bounces) {
        // Importance Sample Cosine Hemisphere
        // Need Random.
        // Use pseudo-random from pixel_index + frame + bounce.
        // Placeholder direction (Reflection)
        let N = normalize(surface.normal);
        let R = reflect(ray.direction, N);

        var next_ray: Ray;
        next_ray.origin = surface.position + N * 0.001;
        next_ray.direction = R; // Should be sampled
        next_ray.depth = ray.depth + 1u;
        next_ray.pixel_index = ray.pixel_index;
        next_ray.throughput = ray.throughput * base_color; // Approximation
        next_ray.t_min = 0.001;
        next_ray.t_max = 10000.0;

        // Compact
        let next_idx = atomicAdd(&counters.next_ray_count, 1u);
        next_rays[next_idx] = next_ray;
    }
}
