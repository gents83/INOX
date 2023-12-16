// Originally taken from https://github.com/KhronosGroup/glTF-WebGL-PBR
// Commit a94655275e5e4e8ae580b1d95ce678b74ab87426
//
// This fragment shader defines a reference implementation for Physically Based Shading of
// a microfacet surface material defined by a glTF model.
//
// References:
// [1] Real Shading in Unreal Engine 4
//     http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
// [2] Physically Based Shading at Disney
//     http://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_notes_v3.pdf
// [3] README.md - Environment Maps
//     https://github.com/KhronosGroup/glTF-WebGL-PBR/#environment-maps
// [4] "An Inexpensive BRDF Model for Physically based Rendering" by Christophe Schlick
//     https://www.cs.virginia.edu/~jdl/bib/appearance/analytic%20models/schlick94b.pdf

const AMBIENT_COLOR: vec3<f32> = vec3<f32>(1., 1., 1.);
const AMBIENT_INTENSITY = 1.;
const NULL_VEC4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
const MIN_ROUGHNESS = 0.04;

// Constant normal incidence Fresnel factor for all dielectrics.
const Fdielectric: vec3<f32> = vec3<f32>(0.04, 0.04, 0.04);
const Epsilon: f32 = 0.00001;

struct PBRData {
    p: vec3<f32>,
    n: vec3<f32>,
    v: vec3<f32>,
    specular_environmentR0: vec3<f32>,
    specular_environmentR90: vec3<f32>,
    diffuse_color: vec3<f32>,
    emissive_color: vec3<f32>,
    alpha_roughness: f32,
    final_color: vec4<f32>,
};

// GGX/Towbridge-Reitz normal distribution function.
// Uses Disney's reparametrization of alpha = roughness^2.
fn ndfGGX(cosLh: f32, roughness: f32) -> f32
{
	let alpha   = roughness * roughness;
	let alphaSq = alpha * alpha;

	let denom = (cosLh * cosLh) * (alphaSq - 1.0) + 1.0;
	return alphaSq / (PI * denom * denom);
}

// Single term for separable Schlick-GGX below.
fn gaSchlickG1(cosTheta: f32, k: f32) -> f32
{
	return cosTheta / (cosTheta * (1.0 - k) + k);
}

// Schlick-GGX approximation of geometric attenuation function using Smith's method.
fn gaSchlickGGX(cosLi: f32, cosLo: f32, roughness: f32) -> f32
{
	let r = roughness + 1.0;
	let k = (r * r) / 8.0; // Epic suggests using this roughness remapping for analytic lights.
	return gaSchlickG1(cosLi, k) * gaSchlickG1(cosLo, k);
}

// Shlick's approximation of the Fresnel factor.
fn fresnelSchlick(F0: vec3<f32>, cosTheta: f32) -> vec3<f32>
{
	return F0 + (vec3(1.0) - F0) * pow(1.0 - cosTheta, 5.0);
}

// The following equation models the Fresnel reflectance term of the spec equation (aka F())
// Implementation of fresnel from [4], Equation 15
fn specular_reflection(reflectance0: vec3<f32>, reflectance90: vec3<f32>, VdotH: f32) -> vec3<f32> {
    return reflectance0 + (reflectance90 - reflectance0) * pow(clamp(1.0 - VdotH, 0.0, 1.0), 5.0);
}
// This calculates the specular geometric attenuation (aka G()),
// where rougher material will reflect less light back to the viewer.
// This implementation is based on [1] Equation 4, and we adopt their modifications to
// alphaRoughness as input as originally proposed in [2].
fn geometric_occlusion(alpha_roughness: f32, NdotL: f32, NdotV: f32) -> f32 {
    let attenuationL = 2.0 * NdotL / (NdotL + sqrt(alpha_roughness * alpha_roughness + (1.0 - alpha_roughness * alpha_roughness) * (NdotL * NdotL)));
    let attenuationV = 2.0 * NdotV / (NdotV + sqrt(alpha_roughness * alpha_roughness + (1.0 - alpha_roughness * alpha_roughness) * (NdotV * NdotV)));
    return attenuationL * attenuationV;
}

// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
fn microfacet_distribution(alpha_roughness: f32, NdotH: f32) -> f32 {
    let roughnessSq = alpha_roughness * alpha_roughness;
    let f = (NdotH * roughnessSq - NdotH) * NdotH + 1.0;
    return roughnessSq / (PI * f * f);
}

fn compute_color(material_id: u32, pixel_data: ptr<function, PixelData>) -> PBRData {
    let material = &materials.data[material_id];

    var diffuse_color = (*pixel_data).color;
    diffuse_color *= (*material).base_color;
    if (has_texture(material_id, TEXTURE_TYPE_BASE_COLOR)) {  
        let uv = material_texture_uv(pixel_data, TEXTURE_TYPE_BASE_COLOR);
        let texture_color = sample_texture(uv);
        diffuse_color *= texture_color;
    }
        
    let f0 = vec3<f32>(0.04);
    var ao = 1.0;
    var occlusion_strength = 1.;

    if (has_texture(material_id, TEXTURE_TYPE_OCCLUSION)) {
        let t = material_texture_uv(pixel_data, TEXTURE_TYPE_OCCLUSION);
        ao = ao * t.r;
        occlusion_strength = (*material).occlusion_strength;
    }

    var emissive_color = (*material).emissive_color;
    if (has_texture(material_id, TEXTURE_TYPE_EMISSIVE)) {
        let t = material_texture_uv(pixel_data, TEXTURE_TYPE_EMISSIVE);
        emissive_color *= t.rgb;
    }

    var perceptual_roughness = (*material).roughness_factor;
    var metallic = (*material).metallic_factor;
    if (has_texture(material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {        
        // Roughness is stored in the 'g' channel, metallic is stored in the 'b' channel.
        // This layout intentionally reserves the 'r' channel for (optional) occlusion map data
        let t = material_texture_uv(pixel_data, TEXTURE_TYPE_METALLIC_ROUGHNESS);
        perceptual_roughness = perceptual_roughness * t.g;
        metallic = metallic * t.b;
    }
    perceptual_roughness = clamp(perceptual_roughness, MIN_ROUGHNESS, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);
    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness [2].
    let alpha_roughness = perceptual_roughness * perceptual_roughness;
    
    // Compute reflectance.
    let specular_color = mix(f0, diffuse_color.rgb, metallic);  
    let reflectance = max(max(specular_color.r, specular_color.g), specular_color.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    let reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    let specular_environmentR0 = specular_color.rgb;
    let specular_environmentR90 = vec3<f32>(1., 1., 1.) * reflectance90;
    
    diffuse_color = vec4<f32>(diffuse_color.rgb * (vec3<f32>(1.) - f0), diffuse_color.a);
    diffuse_color *= (1. - metallic);
        
    var ambient_color = diffuse_color.rgb * AMBIENT_COLOR * AMBIENT_INTENSITY;
    ambient_color = mix(ambient_color, ambient_color * ao, occlusion_strength);
    let color = ambient_color;// + emissive_color;
        
    let final_color = vec4<f32>(color, diffuse_color.a);

    let p = (*pixel_data).world_pos;
    let n = compute_normal(pixel_data);     // normal at surface point
    let view_pos = constant_data.view[3].xyz;
    let v = normalize(view_pos-(*pixel_data).world_pos); // Vector from surface point to camera

    return PBRData(p, n, v, specular_environmentR0, specular_environmentR90, diffuse_color.rgb, emissive_color, alpha_roughness, final_color);
}