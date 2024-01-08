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



// 0-1 linear  from  0-1 sRGB gamma
fn linear_from_gamma_rgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(0.04045);
    let lower = srgb / vec3<f32>(12.92);
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

// 0-1 sRGB gamma  from  0-1 linear
fn gamma_from_linear_rgb(rgb: vec3<f32>) -> vec3<f32> {
    let cutoff = rgb < vec3<f32>(0.0031308);
    let lower = rgb * vec3<f32>(12.92);
    let higher = vec3<f32>(1.055) * pow(rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(higher, lower, cutoff);
}

// 0-1 sRGBA gamma  from  0-1 linear
fn gamma_from_linear_rgba(linear_rgba: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(gamma_from_linear_rgb(linear_rgba.rgb), linear_rgba.a);
}

// [u8; 4] SRGB as u32 -> [r, g, b, a] in 0.-1
fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(color & 255u),
        f32((color >> 8u) & 255u),
        f32((color >> 16u) & 255u),
        f32((color >> 24u) & 255u),
    ) / 255.;
}

// linear to sRGB approximation
// see http://chilliant.blogspot.com/2012/08/srgb-approximations-for-hlsl.html
fn linearTosRGB(color: vec3<f32>) -> vec3<f32>
{
    return pow(color, vec3(INV_GAMMA));
}

// sRGB to linear approximation
// see http://chilliant.blogspot.com/2012/08/srgb-approximations-for-hlsl.html
fn sRGBToLinear(srgbIn: vec3<f32>) -> vec3<f32>
{
    return vec3<f32>(pow(srgbIn.xyz, vec3<f32>(GAMMA)));
}