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

// This is PCG
fn get_random_numbers(seed: vec2<u32>) -> vec2<u32> {
    var new_seed = seed;
    new_seed = 1664525u * new_seed + 1013904223u;
    new_seed.x += 1664525u * new_seed.y;
    new_seed.y += 1664525u * new_seed.x;
    new_seed.x = new_seed.x ^ (new_seed.x >> 16u);
    new_seed.y = new_seed.y ^ (new_seed.y >> 16u);
    new_seed.x += 1664525u * new_seed.y;
    new_seed.y += 1664525u * new_seed.x;
    new_seed.x = new_seed.x ^ (new_seed.x >> 16u);
    new_seed.y = new_seed.y ^ (new_seed.y >> 16u);
    return new_seed;
}

fn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) 
{
    let c = *ptr_a;
    *ptr_a = *ptr_b;
    *ptr_b = c;
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

const GAMMA: f32 = 2.2;
const INV_GAMMA: f32 = 1.0 / GAMMA;
// ODT_SAT => XYZ => D60_2_D65 => sRGB
const ACESOutputMat: mat3x3<f32> = mat3x3<f32>(
    1.60475, -0.10208, -0.00327,
    -0.53108,  1.10813, -0.07276,
    -0.07367, -0.00605,  1.07602
);

fn Uncharted2ToneMapping(color: vec3<f32>) -> vec3<f32> {
	let A = 0.15;
	let B = 0.50;
	let C = 0.10;
	let D = 0.20;
	let E = 0.02;
	let F = 0.30;
	let W = 11.2;
	let exposure = 2.;
	var result = color * exposure;
	result = ((result * (A * result + C * B) + D * E) / (result * (A * result + B) + D * F)) - E / F;
	let white = ((W * (A * W + C * B) + D * E) / (W * (A * W + B) + D * F)) - E / F;
	result /= white;
	result = pow(result, vec3<f32>(1. / GAMMA));
	return result;
}

fn tonemap_ACES_Narkowicz(color: vec3<f32>) -> vec3<f32> {
    let A = 2.51;
    let B = 0.03;
    let C = 2.43;
    let D = 0.59;
    let E = 0.14;
    return clamp((color * (A * color + vec3<f32>(B))) / (color * (C * color + vec3<f32>(D)) + vec3<f32>(E)), vec3<f32>(0.0), vec3<f32>(1.0));
}

// ACES filmic tone map approximation
// see https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
fn RRTAndODTFit(color: vec3<f32>) -> vec3<f32> {
    let a = color * (color + vec3<f32>(0.0245786)) - vec3<f32>(0.000090537);
    let b = color * (0.983729 * color + vec3<f32>(0.4329510)) + vec3<f32>(0.238081);
    return a / b;
}

fn tonemap_ACES_Hill(color: vec3<f32>) -> vec3<f32> {
   var c = ACESOutputMat * RRTAndODTFit(color);
   return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
}
