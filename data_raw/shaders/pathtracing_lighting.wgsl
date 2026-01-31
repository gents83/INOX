#include "raytracing_bindings.inc"

@group(0) @binding(3) var<storage, read> surface_data: array<SurfaceData>;

@group(1) @binding(0) var<storage, read> materials: array<GPUMaterial>; // Define GPUMaterial struct?
@group(1) @binding(1) var textures: binding_array<texture_2d<f32>>; // Bindless? Or array?
@group(1) @binding(2) var<storage, read> lights: array<GPULight>;
@group(1) @binding(3) var<storage, read_write> shadow_rays: array<ShadowRay>;
@group(1) @binding(4) var<storage, read_write> counters: PathTracingCounters;
@group(1) @binding(5) var<storage, read> rays: array<Ray>;
@group(1) @binding(6) var<storage, read_write> next_rays: array<Ray>;
@group(1) @binding(7) var<storage, read_write> accumulator: array<RadiancePackedData>;
@group(1) @binding(8) var sampler_state: sampler;

// Need GPUMaterial and GPULight structs
struct GPUMaterial {
    textures_index_and_coord_set: mat4x4<f32>,
    roughness_factor: f32,
    metallic_factor: f32,
    ior: f32,
    transmission_factor: f32,
    base_color: vec4<f32>,
    emissive_color: vec3<f32>,
    emissive_strength: f32,
    // ...
};
// Simplified for now.

struct GPULight {
    color: vec3<f32>,
    intensity: f32,
    position: vec3<f32>,
    range: f32,
    direction: vec3<f32>,
    inner_cone_cos: f32,
    outer_cone_cos: f32,
    light_type: u32,
};

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

    // Evaluate Material
    // ... (PBR Logic)
    // Assume Diffuse White for test.
    let albedo = vec3<f32>(0.8);

    // NEE
    let num_lights = constant_data.num_lights;
    if (num_lights > 0u) {
        // Sample Light 0
        let light = lights[0];
        let L = normalize(light.position - surface.position);
        let dist = length(light.position - surface.position);

        var shadow_ray: ShadowRay;
        shadow_ray.origin = surface.position + surface.normal * 0.001;
        shadow_ray.direction = L;
        shadow_ray.t_max = dist - 0.002;
        shadow_ray.pixel_index = ray.pixel_index;
        shadow_ray.contribution = albedo * light.color * light.intensity / (dist * dist);
        // Writing to ShadowRays using Atomic Append?
        // Or mapped by index?
        // ShadowRays buffer is sized to screen.
        // We can use pixel_index mapping if we only generate 1 shadow ray.
        // But if we generate 0 (backface), we waste space?
        // Better: Atomic Append for Shadow Rays too!
        // But `ShadowPass` needs dispatch indirect then.
        // For simplicity, map by pixel_index (Dense).
        shadow_rays[ray.pixel_index] = shadow_ray;
        atomicAdd(&counters.shadow_ray_count, 1u); // Just for stats
    }

    // Next Ray (Indirect)
    if (ray.depth < constant_data.num_bounces) {
        // Cosine Sample Hemisphere
        // ...
        var next_ray: Ray;
        next_ray.origin = surface.position + surface.normal * 0.001;
        next_ray.direction = reflect(ray.direction, surface.normal); // Mirror for test
        next_ray.depth = ray.depth + 1u;
        next_ray.pixel_index = ray.pixel_index;
        next_ray.throughput = ray.throughput * albedo;
        next_ray.t_min = 0.001;
        next_ray.t_max = 10000.0;

        // Compact
        let next_idx = atomicAdd(&counters.next_ray_count, 1u);
        next_rays[next_idx] = next_ray;
    }
}
