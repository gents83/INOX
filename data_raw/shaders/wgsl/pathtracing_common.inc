struct Ray {
    origin: vec3<f32>,
    t_min: f32,
    direction: vec3<f32>,
    t_max: f32,
    throughput: vec3<f32>,
    pixel_index: u32,
}

struct RayHit {
    instance_id: u32,
    primitive_index: u32,
    barycentrics: vec2<f32>,
    t: f32,
    pixel_index: u32,
    padding: vec2<u32>,
    direction: vec3<f32>,
    pad_dir: f32,
    throughput: vec3<f32>,
    pad_thr: f32,
}

struct ShadowRay {
    origin: vec3<f32>,
    t_max: f32,
    direction: vec3<f32>,
    radiance: u32,
    contribution: vec3<f32>,
    pixel_index: u32,
}

struct PathTracingCounters {
    ray_count: atomic<u32>,
    hit_count: atomic<u32>,
    shadow_ray_count: atomic<u32>,
    extension_ray_count: atomic<u32>,
}
