struct WavefrontRay {
    origin_and_t_min: vec4<f32>,
    direction_and_t_max: vec4<f32>,
    throughput: vec4<f32>,
    pixel_index: u32,
    bounce: u32,
    flags: u32,
    sample_index: u32,
};

struct WavefrontQueueHeader {
    version: u32,
    count: atomic<u32>,
    capacity: u32,
    overflowed: atomic<u32>,
};

struct WavefrontDispatchArgs {
    x: u32,
    y: u32,
    z: u32,
};

struct WavefrontHit {
    distance: f32,
    visibility_id: u32,
    instance_id: u32,
    primitive_id: u32,
    barycentric: vec2<f32>,
    ray_index: u32,
    flags: u32,
};
