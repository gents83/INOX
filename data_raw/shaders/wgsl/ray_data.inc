struct PathRay {
    origin: vec3<f32>,
    t_min: f32,
    direction: vec3<f32>,
    t_max: f32,
    throughput: vec3<f32>,
    pixel_index: u32,
    ray_type: u32,
    bounce_count: u32,
    flags: u32,
    _padding: u32,
}

struct Intersection {
    t: f32,
    u: f32,
    v: f32,
    instance_id: i32,
    primitive_index: i32,
    padding: u32,
    // Pad to 32 bytes (8 floats) to match Rust IntersectionPackedData
    _pad0: u32, _pad1: u32,
}

struct Rays {
    data: array<PathRay>,
}

struct Intersections {
    data: array<Intersection>,
}

struct MaterialHit {
    albedo: vec4<f32>,
    emission: vec3<f32>,
    roughness: f32,
    metallic: f32,
    ior: f32,
    transmission: f32,
    thickness: f32,
}
