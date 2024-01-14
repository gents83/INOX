fn quantize_unorm(v: f32, n: u32) -> u32 {
    let scale = f32((1u << n) - 1u);
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
    let scale = f32((1u << n) - 1u);
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

fn pack_3_f32_to_unorm(value: vec3<f32>) -> u32 {
    let x = quantize_unorm(value.x, 10u) << 20u;
    let y = quantize_unorm(value.y, 10u) << 10u;
    let z = quantize_unorm(value.z, 10u);
    return (x | y | z);
}
fn unpack_unorm_to_3_f32(v: u32) -> vec3<f32> {
    let vx = decode_unorm((v >> 20u) & 0x000003FFu, 10u);
    let vy = decode_unorm((v >> 10u) & 0x000003FFu, 10u);
    let vz = decode_unorm(v & 0x000003FFu, 10u);
    return vec3<f32>(vx, vy, vz);
}

fn pack_3_f32_to_snorm(value: vec3<f32>) -> u32 {
    let x = quantize_snorm(value.x, 10u) << 20u;
    let y = quantize_snorm(value.y, 10u) << 10u;
    let z = quantize_snorm(value.z, 10u);
    return (x | y | z);
}
fn unpack_snorm_to_3_f32(v: u32) -> vec3<f32> {
    let vx = decode_snorm((v >> 20u) & 0x000003FFu, 10u);
    let vy = decode_snorm((v >> 10u) & 0x000003FFu, 10u);
    let vz = decode_snorm(v & 0x000003FFu, 10u);
    return vec3<f32>(vx, vy, vz);
}

fn unpack_normal(f: f32) -> vec3<f32> {
	var f_var = f;
	var flipZ: f32 = sign(f_var);
	f_var = abs(f_var);
	let atanXY: f32 = floor(f_var) / 67.5501 * (3.1415927 * 2.) - 3.1415927;
	var n: vec3<f32> = vec3<f32>(sin(atanXY), cos(atanXY), 0.);
	n.z = fract(f_var) * 1869.2296 / 427.67993;
	n = normalize(n);
	n.z = n.z * (flipZ);
	return n;
} 

fn pack_normal(n: vec3<f32>) -> f32 {
	var n_var = n;
	let flipZ: f32 = sign(n_var.z);
	n_var.z = abs(n_var.z);
	n_var = n_var / (23.065746);
	let xy: f32 = floor((atan2(n_var.x, n_var.y) + 3.1415927) / (3.1415927 * 2.) * 67.5501);
	var z: f32 = floor(n_var.z * 427.67993) / 1869.2296;
	z = z * (1. / max(0.01, length(vec2<f32>(n_var.x, n_var.y))));
	return (xy + z) * flipZ;
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

fn sign_not_zero(v: vec2<f32>) -> vec2<f32> {
	return vec2<f32>(select(-1., 1., v.x >= 0.), select(-1., 1., v.y >= 0.));
} 

fn octahedral_mapping(v: vec3<f32>) -> vec2<f32> {
	let l1norm: f32 = abs(v.x) + abs(v.y) + abs(v.z);
	var result: vec2<f32> = v.xy * (1. / l1norm);
	if (v.z < 0.) {
		result = (1. - abs(result.yx)) * sign_not_zero(result.xy);
	}
	return result;
} 

fn octahedral_unmapping(o: vec2<f32>) -> vec3<f32> {
	var v: vec3<f32> = vec3<f32>(o.x, o.y, 1. - abs(o.x) - abs(o.y));
	if (v.z < 0.) {
		var vxy = v.xy;
        vxy = (1. - abs(v.yx)) * sign_not_zero(v.xy);
        v.x = vxy.x;
        v.y = vxy.y;
	}
	return normalize(v);
} 

fn f32tof16(v: f32) -> u32 {
    return pack2x16float(vec2<f32>(v, 0.));
}

fn f16tof32(v: u32) -> f32 {
    return unpack2x16float(v & 0x0000FFFFu).x;
}

fn pack_into_R11G11B10F(rgb: vec3<f32>) -> u32 {
	let r = (f32tof16(rgb.r) << 17u) & 0xFFE00000u;
	let g = (f32tof16(rgb.g) << 6u) & 0x001FFC00u;
	let b = (f32tof16(rgb.b) >> 5u) & 0x000003FFu;
	return r | g | b;
} 

fn unpack_from_R11G11B10F(rgb: u32) -> vec3<f32> {
	let r = f16tof32((rgb >> 17u) & 0x7FF0u);
	let g = f16tof32((rgb >> 6u) & 0x7FF0u);
	let b = f16tof32((rgb << 5u) & 0x7FE0u);
	return vec3<f32>(r, g, b);
} 


fn iq_hash(v: vec2<f32>) -> f32 {
    return fract(sin(dot(v, vec2(11.9898, 78.233))) * 43758.5453);
}
fn blue_noise(in: vec2<f32>) -> f32 {
    var v =  iq_hash( in + vec2<f32>(-1., 0.) )
             + iq_hash( in + vec2<f32>( 1., 0.) )
             + iq_hash( in + vec2<f32>( 0., 1.) )
             + iq_hash( in + vec2<f32>( 0.,-1.) ); 
    v /= 4.;
    return (iq_hash(in) - v + .5);
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

fn hash1(seed: f32) -> f32 {
    var p = fract(seed * .1031);
    p *= p + 33.33;
    p *= p + p;
    return fract(p);
}

fn hash2(seed: ptr<function, f32>) -> vec2<f32> {
    let a = (*seed) + 0.1;
    let b = a + 0.1;
    (*seed) = b;
    return fract(sin(vec2(a,b))*vec2(43758.5453123,22578.1459123));
}

fn hash3(seed: ptr<function, f32>) -> vec3<f32> {
    let a = (*seed) + 0.1;
    let b = a + 0.1;
    let c = b + 0.1;
    (*seed) = c;
    return fract(sin(vec3(a,b,c))*vec3(43758.5453123,22578.1459123,19642.3490423));
}

// This is PCG2d
fn get_random_numbers(seed: ptr<function, vec2<u32>>) -> vec2<f32> {
    var v = (*seed) * 1664525u + 1013904223u;
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    *seed = v;
    return vec2<f32>(v) * 2.32830643654e-10;
}

fn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) 
{
    let c = *ptr_a;
    *ptr_a = *ptr_b;
    *ptr_b = c;
}

fn mod_f32(v: f32, m: f32) -> f32
{
    return v - (m * floor(v/m));
}

fn clamped_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return clamp(dot(a,b), 0., 1.);
}

fn has_vertex_attribute(vertex_attribute_layout: u32, attribute_to_check: u32) -> bool {
    return bool(vertex_attribute_layout & attribute_to_check);
}
fn vertex_attribute_offset(vertex_attribute_layout: u32, attribute_to_check: u32) -> i32 
{
    if(has_vertex_attribute(vertex_attribute_layout, attribute_to_check)) {
        let mask = (vertex_attribute_layout & 0x0000FFFFu) & (~attribute_to_check & (attribute_to_check - 1u));
        return i32(countOneBits(mask));
    }
    return -1;
}
fn vertex_layout_stride(vertex_attribute_layout: u32) -> u32 
{
    return countOneBits((vertex_attribute_layout & 0x0000FFFFu));
}
