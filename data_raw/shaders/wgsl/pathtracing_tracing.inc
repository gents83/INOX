// BVH-dependent tracing utilities
// Only include this file in shaders that also include raytracing.inc (which provides traverse_bvh)

// Trace a shadow ray - returns 1.0 if lit, 0.0 if in shadow
fn trace_shadow_ray(
    origin: vec3<f32>,
    to_light: vec3<f32>,
    max_distance: f32,
    tlas_starting_index: u32
) -> f32 {
    let direction = normalize(to_light);
    let shadow_origin = origin + direction * HIT_EPSILON;
    let result = traverse_bvh(shadow_origin, direction, tlas_starting_index);

    if (result.visibility_id != 0u && (result.visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
        if (result.distance < max_distance) {
            return 0.;
        }
    }
    return 1.;
}

// Trace an AO ray - returns 1.0 if no occlusion, 0.0 if occluded
fn trace_ao_ray(
    origin: vec3<f32>,
    direction: vec3<f32>,
    ao_radius: f32,
    tlas_starting_index: u32
) -> f32 {
    let ao_origin = origin + direction * HIT_EPSILON;
    let result = traverse_bvh(ao_origin, direction, tlas_starting_index);

    if (result.visibility_id != 0u && (result.visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
        if (result.distance < ao_radius) {
            return 0.;
        }
    }
    return 1.;
}
