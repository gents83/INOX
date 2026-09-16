const WAVEFRONT_RAY_QUEUE_RECORD_STRIDE_WORDS: u32 = 16u;
const WAVEFRONT_HIT_RECORD_STRIDE_WORDS: u32 = 8u;
const WAVEFRONT_ACCUMULATION_RECORD_STRIDE_WORDS: u32 = 8u;
const WAVEFRONT_RESERVOIR_RECORD_STRIDE_WORDS: u32 = 8u;
const WAVEFRONT_AO_RECORD_STRIDE_WORDS: u32 = 4u;
const WAVEFRONT_TRANSPARENT_HIT_RECORD_STRIDE_WORDS: u32 = 10u;
const WAVEFRONT_DISPATCH_ARGS_STRIDE_WORDS: u32 = 3u;
const WAVEFRONT_QUEUE_HEADER_STRIDE_WORDS: u32 = 4u;
const PACKED_LIGHT_ALIAS_VERSION: u32 = 1u;
const PACKED_LIGHT_ALIAS_HEADER_WORDS: u32 = 4u;
const PACKED_LIGHT_ALIAS_RECORD_STRIDE_WORDS: u32 = 4u;
const PACKED_LIGHT_ALIAS_FLAG_OVERFLOW: u32 = 1u;

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
