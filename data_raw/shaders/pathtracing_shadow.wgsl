#include "raytracing_bindings.inc"

@group(1) @binding(0) var<storage, read> shadow_rays: array<ShadowRay>;
@group(1) @binding(1) var<storage, read_write> accumulator: array<RadiancePackedData>;
@group(1) @binding(2) var<storage, read> counters: PathTracingCounters;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    // How many shadow rays?
    // If dense, use screen resolution.
    // If we used dense mapping in Lighting, we iterate all pixels.
    // Or we use a counter if we compacted.
    // I used Dense mapping in Lighting (`shadow_rays[ray.pixel_index]`).
    // So dispatch is screen size.
    // But `ComputePathTracingShadowPass` usually dispatches dense.

    let resolution = vec2<u32>(u32(constant_data.screen_size.x), u32(constant_data.screen_size.y));
    let pixel_index = index;
    if (pixel_index >= resolution.x * resolution.y) {
        return;
    }

    let shadow_ray = shadow_rays[pixel_index];
    if (length(shadow_ray.contribution) <= 0.0) {
        return;
    }

    let visible = !traverse_bvh_shadow(shadow_ray);

    if (visible) {
        // Accumulate
        // RadiancePackedData is f32 (packed?).
        // If it's vec4, use vec4.
        // Assuming accumulator stores RGB (vec3) + Count?
        // Or RGBA?
        // Rust struct: `RadiancePackedData(f32)`.
        // This implies single channel? Or placeholder?
        // Path Tracer usually accumulates RGB.
        // I should change `RadiancePackedData` to `vec4<f32>`.
        // But for now, assuming float accumulation (Grey).
        // Or maybe `accumulator` is `array<vec4<f32>>` but typed as `RadiancePackedData` in Rust?
        // I'll assume `vec4`.
        // But `RadiancePackedData` struct in WGSL is needed.
        // `raytracing_structs.inc` defined `RadiancePackedData`? No.
        // I need to add it.
    }
}
