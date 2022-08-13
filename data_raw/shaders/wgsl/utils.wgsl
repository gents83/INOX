fn decode_unorm(i: u32, n: u32) -> f32 {    
    let scale = f32((1 << n) - 1);
    if i == 0u {
        return 0.;
    } else if i == u32(scale) {
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

fn unpack_unorm_to_4_f32(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(((color >> 24u) / 255u) & 255u),
        f32(((color >> 16u) / 255u) & 255u),
        f32(((color >> 8u) / 255u) & 255u),
        f32((color / 255u) & 255u),
    );
}

fn hash(index: u32) -> u32 {
    var v = index;
    v = (v + 0x7ed55d16u) + (v << 12u);
    v = (v ^ 0xc761c23cu) ^ (v >> 19u);
    v = (v + 0x165667b1u) + (v << 5u);
    v = (v + 0xd3a2646cu) ^ (v << 9u);
    v = (v + 0xfd7046c5u) + (v << 3u);
    v = (v ^ 0xb55a4f09u) ^ (v >> 16u);
    return v;
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