fn quantize_unorm(v: f32, n: u32) -> u32 {
    let scale = f32((1 << n) - 1);
    return u32(0.5 + (v * scale));
}
fn quantize_snorm(v: f32, n: u32) -> u32 {
    let c = (1u << (n - 1u)) - 1u;
    let scale = f32(c);
    if v < 0. {
        return (u32(-v * scale) & c) | (1u << (n - 1u));
    } else {
        return u32(v * scale) & c;
    }
}

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

fn decode_snorm(i: u32, n: u32) -> f32 {
    let s = i >> (n - 1u);
    let c = (1u << (n - 1u)) - 1u;
    let scale = f32(c);
    if s > 0u {
        let r = f32(i & c) / scale;
        return -r;
    } else {
        return f32(i & c) / scale;
    }
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

fn pack_4_f32_to_unorm(value: vec4<f32>) -> u32 {
    let r = quantize_unorm(value.x, 8u) << 24u;
    let g = quantize_unorm(value.y, 8u) << 16u;
    let b = quantize_unorm(value.z, 8u) << 8u;
    let a = quantize_unorm(value.w, 8u);
    return (r | g | b | a);
}
fn unpack_snorm_to_4_f32(v: u32) -> vec4<f32> {
    let r = decode_snorm((v >> 24u) & 255u, 8u);
    let g = decode_snorm((v >> 16u) & 255u, 8u);
    let b = decode_snorm((v >> 8u) & 255u, 8u);
    let a = decode_snorm(v & 255u, 8u);
    return vec4<f32>(r,g,b,a);
}
fn unpack_unorm_to_4_f32(v: u32) -> vec4<f32> {
    let r = decode_unorm((v >> 24u) & 255u, 8u);
    let g = decode_unorm((v >> 16u) & 255u, 8u);
    let b = decode_unorm((v >> 8u) & 255u, 8u);
    let a = decode_unorm(v & 255u, 8u);
    return vec4<f32>(r,g,b,a);
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

fn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) 
{
    let c = *ptr_a;
    *ptr_a = *ptr_b;
    *ptr_b = c;
}

fn has_vertex_attribute(vertex_attribute_layout: u32, attribute_to_check: u32) -> bool {
    return bool(vertex_attribute_layout & attribute_to_check);
}
fn vertex_attribute_offset(vertex_attribute_layout: u32, attribute_to_check: u32) -> u32 
{
    var result = 0u;
    if(attribute_to_check >= VERTEX_ATTRIBUTE_HAS_COLOR && has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_COLOR)) {
        result += 1u;
    }
    if(attribute_to_check >= VERTEX_ATTRIBUTE_HAS_NORMAL && has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_NORMAL)) {
        result += 1u;
    }
    if(attribute_to_check >= VERTEX_ATTRIBUTE_HAS_UV1 && has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_UV1)) {
        result += 1u;
    }
    if(attribute_to_check >= VERTEX_ATTRIBUTE_HAS_UV2 && has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_UV2)) {
        result += 1u;
    }
    if(attribute_to_check >= VERTEX_ATTRIBUTE_HAS_UV3 && has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_UV3)) {
        result += 1u;
    }
    if(attribute_to_check >= VERTEX_ATTRIBUTE_HAS_UV4 && has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_UV4)) {
        result += 1u;
    }
    return result;
}
fn vertex_layout_stride(vertex_attribute_layout: u32) -> u32 
{
    var result = 0u;
    if(has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_COLOR)) {
        result += 1u;
    }
    if(has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_NORMAL)) {
        result += 1u;
    }
    if(has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_UV1)) {
        result += 1u;
    }
    if(has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_UV2)) {
        result += 1u;
    }
    if(has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_UV3)) {
        result += 1u;
    }
    if(has_vertex_attribute(vertex_attribute_layout, VERTEX_ATTRIBUTE_HAS_UV4)) {
        result += 1u;
    }
    return result;
}