
let PI: f32 = 3.141592653589793;
let AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.75, 0.75, 0.75);
let AMBIENT_INTENSITY = 0.45;
let NULL_VEC4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
let MIN_ROUGHNESS = 0.04;

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
    let r = alpha_roughness;

    let attenuationL = 2.0 * NdotL / (NdotL + sqrt(r * r + (1.0 - r * r) * (NdotL * NdotL)));
    let attenuationV = 2.0 * NdotV / (NdotV + sqrt(r * r + (1.0 - r * r) * (NdotV * NdotV)));
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

fn pbr(world_pos: vec3<f32>, n: vec3<f32>, material_id: u32, color: vec4<f32>, uvs_0_1: vec4<f32>,) -> vec4<f32> {
    let material = &materials.data[material_id];
    var perceptual_roughness = (*material).roughness_factor;
    var metallic = (*material).metallic_factor;
    if (has_texture(material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {
        let t = sample_material_texture(uvs_0_1, material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS);
        metallic = metallic * t.b;
        perceptual_roughness = perceptual_roughness * t.g;
    }
    perceptual_roughness = clamp(perceptual_roughness, MIN_ROUGHNESS, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);
    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness [2].
    let alpha_roughness = perceptual_roughness * perceptual_roughness;

    var ao = 1.0;
    var occlusion_strength = 1.;
    if (has_texture(material_id, TEXTURE_TYPE_OCCLUSION)) {
        let t = sample_material_texture(uvs_0_1, material_id, TEXTURE_TYPE_OCCLUSION);
        ao = ao * t.r;
        occlusion_strength = (*material).occlusion_strength;
    }
    var emissive_color = vec3<f32>(0., 0., 0.);
    if (has_texture(material_id, TEXTURE_TYPE_EMISSIVE)) {
        let t = sample_material_texture(uvs_0_1, material_id, TEXTURE_TYPE_EMISSIVE);
        emissive_color = t.rgb * (*material).emissive_color;
    }

    let f0 = vec3<f32>(0.04, 0.04, 0.04);
    var diffuse_color = color.rgb * (vec3<f32>(1., 1., 1.) - f0);
    diffuse_color = diffuse_color * (1.0 - metallic);
    let specular_color = mix(f0, color.rgb, metallic);        

    // Compute reflectance.
    let reflectance = max(max(specular_color.r, specular_color.g), specular_color.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    let reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    let specular_environmentR0 = specular_color.rgb;
    let specular_environmentR90 = vec3<f32>(1., 1., 1.) * reflectance90;

    let view_pos = constant_data.view[3].xyz;
    let v = normalize(view_pos-world_pos);

    let NdotV = clamp(abs(dot(n, v)), 0.0001, 1.0);
    let reflection = reflect(-v, n);
    
    var final_color = color.rgb * AMBIENT_COLOR * AMBIENT_INTENSITY;
    final_color = mix(final_color, final_color * ao, occlusion_strength);
    final_color = final_color + emissive_color;

    let num_lights = arrayLength(&lights.data);
    for (var i = 0u; i < num_lights; i++ ) {
        let light = &lights.data[i];
        if ((*light).light_type == 0u) {
            break;
        }
        let dir = (*light).position - world_pos;
        let l = normalize(dir);                 // Vector from surface point to light
        let h = normalize(l + v);               // Half vector between both l and v
        let dist = length(dir);                 // Distance from surface point to light

        let NdotL = clamp(dot(n, l), 0.0001, 1.0);
        let NdotH = clamp(dot(n, h), 0.0, 1.0);
        let LdotH = clamp(dot(l, h), 0.0, 1.0);
        let VdotH = clamp(dot(v, h), 0.0, 1.0);
        
        // Calculate the shading terms for the microfacet specular shading model
        let F = specular_reflection(specular_environmentR0, specular_environmentR90, VdotH);
        let G = geometric_occlusion(alpha_roughness, NdotL, NdotV);
        let D = microfacet_distribution(alpha_roughness, NdotH);

        // Calculation of analytical lighting contribution
        var intensity = max(200., (*light).intensity);
        intensity = intensity / (4. * PI);
        let range = max(8., (*light).range);
        let light_contrib = (1. - min(dist / range, 1.)) * intensity;
        let diffuse_contrib = (1. - F) * diffuse_color / PI;
        let spec_contrib = F * G * D / (4.0 * NdotL * NdotV);
        var light_color = NdotL * (*light).color.rgb * (diffuse_contrib + spec_contrib);
        light_color = light_color * light_contrib;

        final_color = final_color + light_color;
    }
    
    return vec4<f32>(final_color, color.a);
}