fn decode_unorm(i: u32, n: u32) -> f32 {    
    let scale = f32((1 << n) - 1);
    if (i == 0u) {
        return 0.;
    } else if (i == u32(scale)) {
        return 1.;
    } else {
        return (f32(i) - 0.5) / scale;
    }
}

fn decode_snorm(i: i32, n: u32) -> f32 {
    let scale = f32(1 << (n - 1u));
    return (f32(i) / scale);
}


fn decode_uv(v: u32) -> vec2<f32> {
    return unpack2x16float(v);
}
fn decode_as_vec3(v: u32) -> vec3<f32> {
    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);
    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);
    let vz = decode_unorm(v & 0x000003FFu, 10u);
    return vec3<f32>(vx, vy, vz);
}

fn pack_normal(normal: vec3<f32>) -> vec2<f32> {
    return vec2<f32>(normal.xy * 0.5 + 0.5);
}
fn unpack_normal(uv: vec2<f32>) -> vec3<f32> {
    return vec3<f32>(uv.xy * 2. - 1., sqrt(1.-dot(uv.xy, uv.xy)));
}

fn quantize_unorm(v: f32, n: u32) -> u32 {
    let scale = f32((1 << n) - 1);
    return u32(0.5 + (v * scale));
}

fn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {
    let r = quantize_unorm(value.x, 8u) << 24u;
    let g = quantize_unorm(value.y, 8u) << 16u;
    let b = quantize_unorm(value.z, 8u) << 8u;
    let a = quantize_unorm(value.w, 8u);
    return (r | g | b | a);
}
fn unpack_snorm_to_4_f32(v: i32) -> vec4<f32> {
    return vec4<f32>(
        f32((v >> 24u) & 255) / 255.,
        f32((v >> 16u) & 255) / 255.,
        f32((v >> 8u) & 255) / 255.,
        f32(v & 255) / 255.,
    );
}
fn unpack_unorm_to_4_f32(v: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((v >> 24u) & 255u) / 255.,
        f32((v >> 16u) & 255u) / 255.,
        f32((v >> 8u) & 255u) / 255.,
        f32(v & 255u) / 255.,
    );
}

// 0-1 from 0-255
fn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(10.31475);
    let lower = srgb / vec3<f32>(3294.6);
    let higher = pow((srgb + vec3<f32>(14.025)) / vec3<f32>(269.025), vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

// [u8; 4] SRGB as u32 -> [r, g, b, a]
fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(color & 255u),
        f32((color >> 8u) & 255u),
        f32((color >> 16u) & 255u),
        f32((color >> 24u) & 255u),
    );
}

fn extract_scale(m: mat4x4<f32>) -> vec3<f32> {
    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
    let sx = length(s[0]);
    let sy = length(s[1]);
    let det = determinant(s);
    var sz = length(s[2]);
    if (det < 0.) {
        sz = -sz;
    }
    return vec3<f32>(sx, sy, sz);
}

fn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> {
    if (row == 1u) {
        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);
    } else if (row == 2u) {
        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);
    } else if (row == 3u) {
        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);
    } else {        
        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);
    }
}

fn normalize_plane(plane: vec4<f32>) -> vec4<f32> {
    return (plane / length(plane.xyz));
}

fn rotate_vector(v: vec3<f32>, orientation: vec4<f32>) -> vec3<f32> {
    return v + 2. * cross(orientation.xyz, cross(orientation.xyz, v) + orientation.w * v);
}
fn transform_vector(v: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> vec3<f32> {
    return rotate_vector(v, orientation) * scale + position;
}

// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
fn hash( x: u32 ) -> u32 {
    var v = x;
    v += ( v << 10u );
    v ^= ( v >>  6u );
    v += ( v <<  3u );
    v ^= ( v >> 11u );
    v += ( v << 15u );
    return v;
}