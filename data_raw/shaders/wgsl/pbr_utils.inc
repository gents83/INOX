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

let PI: f32 = 3.141592653589793;
let AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.75, 0.75, 0.75);
let AMBIENT_INTENSITY = 0.25;
let NULL_VEC4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
let MIN_ROUGHNESS = 0.04;

// Constant normal incidence Fresnel factor for all dielectrics.
let Fdielectric: vec3<f32> = vec3<f32>(0.04, 0.04, 0.04);
let Epsilon: f32 = 0.00001;

fn compute_alpha(material_index: u32, vertex_color_alpha: f32) -> f32 {
    let material = &materials.data[material_index];
    // NOTE: the spec mandates to ignore any alpha value in 'OPAQUE' mode
    var alpha = 1.;
    if ((*material).alpha_mode == MATERIAL_ALPHA_BLEND_OPAQUE) {
        alpha = 1.;
    } else if ((*material).alpha_mode == MATERIAL_ALPHA_BLEND_MASK) {
        if (alpha >= (*material).alpha_cutoff) {
            // NOTE: If rendering as masked alpha and >= the cutoff, render as fully opaque
            alpha = 1.;
        } else {
            // NOTE: output_color.a < material.alpha_cutoff should not is not rendered
            // NOTE: This and any other discards mean that early-z testing cannot be done!
            alpha = -1.;
        }
    } else if ((*material).alpha_mode == MATERIAL_ALPHA_BLEND_BLEND) {
        alpha = min((*material).base_color.a, vertex_color_alpha);
    }
    return alpha;
}

fn compute_normal(material_id: u32, normal: vec3<f32>, position: vec3<f32>, uv_set: vec4<u32>) -> vec3<f32> {
    let material = &materials.data[material_id];
    var n = normal;
    //if (has_texture(material_id, TEXTURE_TYPE_NORMAL)) {    
    //    let uv = compute_uvs(material_id, TEXTURE_TYPE_NORMAL, uv_set);    
    //    //// get edge vectors of the pixel triangle 
    //    let dp1 = dpdx( position ); 
    //    let dp2 = dpdy( position ); 
    //    let duv1 = dpdx( uv.xy ); 
    //    let duv2 = dpdy( uv.xy );   
    //    // solve the linear system 
    //    let dp2perp = cross( dp2, n ); 
    //    let dp1perp = cross( n, dp1 ); 
    //    let tangent = dp2perp * duv1.x + dp1perp * duv2.x; 
    //    let bitangent = dp2perp * duv1.y + dp1perp * duv2.y;
    //    let t = normalize(tangent);
    //    let b = normalize(bitangent); 
    //    let tbn = mat3x3<f32>(t, b, n);
    //    let tn = sample_texture(uv);
    //    n = tbn * (2.0 * tn.rgb - vec3<f32>(1.));
    //    n = normalize(n);
    //}
    return n;
}

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

fn compute_brdf(world_pos: vec3<f32>, normal: vec3<f32>, material_id: u32, color: vec4<f32>, uv_set: vec4<u32>) -> vec4<f32> {
    let material = &materials.data[material_id];
    var perceptual_roughness = (*material).roughness_factor;
    var metallic = (*material).metallic_factor;
    if (has_texture(material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {        
        // Roughness is stored in the 'g' channel, metallic is stored in the 'b' channel.
        // This layout intentionally reserves the 'r' channel for (optional) occlusion map data
        let t = sample_material_texture(material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS, uv_set);
        perceptual_roughness = perceptual_roughness * t.g;
        metallic = metallic * t.b;
    }
    perceptual_roughness = clamp(perceptual_roughness, MIN_ROUGHNESS, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);
    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness [2].
    let alpha_roughness = perceptual_roughness * perceptual_roughness;

    var ao = 1.0;
    var occlusion_strength = 1.;
    if (has_texture(material_id, TEXTURE_TYPE_OCCLUSION)) {
        let t = sample_material_texture(material_id, TEXTURE_TYPE_OCCLUSION, uv_set);
        ao = ao * t.r;
        occlusion_strength = (*material).occlusion_strength;
    }
    var emissive_color = vec3<f32>(0.);
    if (has_texture(material_id, TEXTURE_TYPE_EMISSIVE)) {
        let t = sample_material_texture(material_id, TEXTURE_TYPE_EMISSIVE, uv_set);
        emissive_color = t.rgb * (*material).emissive_color;
    }

    let f0 = vec3<f32>(0.04);
    var diffuse_color = color.rgb * (vec3<f32>(1.) - f0);
    diffuse_color = diffuse_color * (1. - metallic);
    let specular_color = mix(f0, color.rgb, metallic);        

    // Compute reflectance.
    let reflectance = max(max(specular_color.r, specular_color.g), specular_color.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    let reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    let specular_environmentR0 = specular_color.rgb;
    let specular_environmentR90 = vec3<f32>(1., 1., 1.) * reflectance90;

    let n = compute_normal(material_id, normal, world_pos, uv_set);     // normal at surface point
    let view_pos = constant_data.view[3].xyz;
    let v = normalize(view_pos-world_pos);                      // Vector from surface point to camera

    let NdotV = clamp(abs(dot(n, v)), 0.0001, 1.0);
    let reflection = reflect(-v, n);
    
    var ambient_color = color.rgb * AMBIENT_COLOR * AMBIENT_INTENSITY;
    ambient_color = mix(ambient_color, ambient_color * ao, occlusion_strength);
    var final_color = ambient_color + emissive_color;

    let num_lights = arrayLength(&lights.data);
    for (var i = 0u; i < num_lights; i++ ) {
        let light = &lights.data[i];
        if ((*light).light_type == 0u) {
            break;
        }

        let dir = (*light).position - world_pos;
        let d = length(dir);
        let l = normalize(dir);                             // Vector from surface point to light
        let h = normalize(l + v);                           // Half vector between both l and v

        let linear_att = 0.5 * d;
        let quad_att = 0.5 * d * d;
        let light_intensity = (*light).intensity * 1. / (linear_att * quad_att);
        let light_contrib = light_intensity * (max((*light).range - d, (*light).range) / (*light).range);
        
        let NdotL = clamp(dot(n, l), 0.0001, 1.0);
        let NdotH = clamp(dot(n, h), 0.0, 1.0);
        let LdotH = clamp(dot(l, h), 0.0, 1.0);
        let VdotH = clamp(dot(v, h), 0.0, 1.0);
        
        // Calculate the shading terms for the microfacet specular shading model
        let F = specular_reflection(specular_environmentR0, specular_environmentR90, VdotH);
        let G = geometric_occlusion(alpha_roughness, NdotL, NdotV);
        let D = microfacet_distribution(alpha_roughness, NdotH);

        let diffuse_contrib = (1. - F) * diffuse_color / PI;
        let spec_contrib = F * G * D / (4.0 * NdotL * NdotV);
        var light_color = NdotL * (*light).color.rgb * (diffuse_contrib + spec_contrib);
        
        final_color = final_color + light_color * light_contrib;
    }
    
    return vec4<f32>(final_color, color.a);
}