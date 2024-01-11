// Inspired from https://github.com/KhronosGroup/glTF-Sample-Viewer

// XYZ to sRGB color space
const XYZ_TO_REC709: mat3x3<f32> = mat3x3<f32>(
     3.2404542, -0.9692660,  0.0556434,
    -1.5371385,  1.8760108, -0.2040259,
    -0.4985314,  0.0415560,  1.0572252
);

// Assume air interface for top
// Note: We don't handle the case fresnel0 == 1
fn fresnel0_to_ior(fresnel0: vec3<f32>) -> vec3<f32> {
    let sqrtF0 = sqrt(fresnel0);
    return (vec3<f32> (1.0) + sqrtF0) / (vec3<f32> (1.0) - sqrtF0);
}

// Conversion FO/IOR
fn ior_to_fresnel0_vec3(transmittedIor: vec3<f32>, incidentIor: f32) -> vec3<f32>{
    let ior = (transmittedIor - vec3<f32>(incidentIor)) / (transmittedIor + vec3<f32>(incidentIor));
    return ior * ior;
}

// ior is a value between 1.0 and 3.0. 1.0 is air interface
fn ior_to_fresnel0_f32(transmittedIor: f32, incidentIor: f32) -> f32 {
    let ior = (transmittedIor - incidentIor) / (transmittedIor + incidentIor);
    return ior * ior;
}

fn apply_ior_to_roughness(roughness: f32, ior: f32) -> f32 {
    // Scale roughness with IOR so that an IOR of 1.0 results in no microfacet refraction and
    // an IOR of 1.5 results in the default amount of microfacet refraction.
    return roughness * clamp(ior * 2.0 - 2.0, 0.0, 1.0);
}

fn f_schlick_f32(f0: f32, f90: f32, VdotH: f32) -> f32 {
    let x = clamp(1.0 - VdotH, 0.0, 1.0);
    let x2 = x * x;
    let x5 = x * x2 * x2;
    return f0 + (f90 - f0) * x5;
}

fn f_schlick_vec3_f32(f0: vec3<f32>, f90: f32, VdotH: f32) -> vec3<f32>
{
    let x = clamp(1.0 - VdotH, 0.0, 1.0);
    let x2 = x * x;
    let x5 = x * x2 * x2;
    return f0 + (f90 - f0) * x5;
}

fn f_schlick_vec3_vec3(f0: vec3<f32>, f90: vec3<f32>, VdotH: f32) -> vec3<f32>
{
    return f0 + (f90 - f0) * pow(clamp(1.0 - VdotH, 0.0, 1.0), 5.0);
}

fn schlick_to_f0_vec3(f: vec3<f32>, f90: vec3<f32>, VdotH: f32) -> vec3<f32> {
    let x = clamp(1.0 - VdotH, 0.0, 1.0);
    let x2 = x * x;
    let x5 = clamp(x * x2 * x2, 0.0, 0.9999);
    let r = (f - (f90 * x5));
    return (r / (1.0 - x5));
}

// Smith Joint GGX
// Note: Vis = G / (4 * NdotL * NdotV)
// see Eric Heitz. 2014. Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs. Journal of Computer Graphics Techniques, 3
// see Real-Time Rendering. Page 331 to 336.
// see https://google.github.io/filament/Filament.md.html#materialsystem/specularbrdf/geometricshadowing(specularg)
fn V_GGX(NdotL: f32, NdotV: f32, alphaRoughness: f32) -> f32 {
    let alphaRoughnessSq = alphaRoughness * alphaRoughness;

    let GGXV = NdotL * sqrt(NdotV * NdotV * (1.0 - alphaRoughnessSq) + alphaRoughnessSq);
    let GGXL = NdotV * sqrt(NdotL * NdotL * (1.0 - alphaRoughnessSq) + alphaRoughnessSq);

    let GGX = GGXV + GGXL;
    return select(0., 0.5 / GGX, GGX > 0.0);
}

// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
fn D_GGX(NdotH: f32, alphaRoughness: f32) -> f32 {
    let alphaRoughnessSq = alphaRoughness * alphaRoughness;
    let f = (NdotH * NdotH) * (alphaRoughnessSq - 1.0) + 1.0;
    return alphaRoughnessSq / (MATH_PI * f * f);
}

// GGX Mask/Shadowing Anisotropic (Same as Babylon.js - smithVisibility_GGXCorrelated_Anisotropic)
// Heitz http://jcgt.org/published/0003/02/03/paper.pdf
fn V_GGX_anisotropic(NdotL: f32, NdotV: f32, BdotV: f32, TdotV: f32, TdotL: f32, BdotL: f32, at: f32, ab: f32) -> f32 {
    let GGXV = NdotL * length(vec3(at * TdotV, ab * BdotV, NdotV));
    let GGXL = NdotV * length(vec3(at * TdotL, ab * BdotL, NdotL));
    let v = 0.5 / (GGXV + GGXL);
    return clamp(v, 0.0, 1.0);
}

// GGX Distribution Anisotropic (Same as Babylon.js)
// https://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_notes_v3.pdf Addenda
fn D_GGX_anisotropic(NdotH: f32, TdotH: f32, BdotH: f32, anisotropy: f32, at: f32, ab: f32) -> f32 {
    let a2 = at * ab;
    let f = vec3(ab * TdotH, at * BdotH, a2 * NdotH);
    let w2 = a2 / dot(f, f);
    return a2 * w2 * w2 / MATH_PI;
}

// Estevez and Kulla http://www.aconty.com/pdf/s2017_pbs_imageworks_sheen.pdf
fn D_Charlie(sheenRoughness: f32, NdotH: f32) -> f32 {
    let s = max(sheenRoughness, 0.000001); //clamp (0,1)
    let alphaG = s * s;
    let invR = 1.0 / alphaG;
    let cos2h = NdotH * NdotH;
    let sin2h = 1.0 - cos2h;
    return (2.0 + invR) * pow(sin2h, invR * 0.5) / (2.0 * MATH_PI);
}

fn lambda_sheen_numeric_helper(x: f32, alphaG: f32) -> f32 {
    let oneMinusAlphaSq = (1.0 - alphaG) * (1.0 - alphaG);
    let a = mix(21.5473, 25.3245, oneMinusAlphaSq);
    let b = mix(3.82987, 3.32435, oneMinusAlphaSq);
    let c = mix(0.19823, 0.16801, oneMinusAlphaSq);
    let d = mix(-1.97760, -1.27393, oneMinusAlphaSq);
    let e = mix(-4.32054, -4.85967, oneMinusAlphaSq);
    return a / (1.0 + b * pow(x, c)) + d * x + e;
}

fn lambda_sheen(cosTheta: f32, alphaG: f32) -> f32 {
    if (abs(cosTheta) < 0.5)
    {
        return exp(lambda_sheen_numeric_helper(cosTheta, alphaG));
    }
    else
    {
        return exp(2.0 * lambda_sheen_numeric_helper(0.5, alphaG) - lambda_sheen_numeric_helper(1.0 - cosTheta, alphaG));
    }
}

fn V_Sheen(NdotL: f32, NdotV: f32, sheenRoughness: f32) -> f32 {
    let s = max(sheenRoughness, 0.000001); //clamp (0,1)
    let alphaG = sheenRoughness * sheenRoughness;

    return clamp(1.0 / ((1.0 + lambda_sheen(NdotV, alphaG) + lambda_sheen(NdotL, alphaG)) *
        (4.0 * NdotV * NdotL)), 0.0, 1.0);
}

fn sheen_LUT(uv: vec2<f32>) -> vec4<f32> {
    let tl = vec4<f32>(1.0,1.0,1.0,1.);
    let tr = vec4<f32>(0.0,0.0,0.0,1.);
    let bl = vec4<f32>(1.0,1.0,1.0,1.);
    let br = vec4<f32>(0.5,0.5,0.5,1.);
    return mix(mix(tr,tr,uv.x),mix(bl,br,uv.x),uv.y);
}

fn albedo_sheen_scaling_LUT(NdotV: f32, sheenRoughnessFactor: f32) -> f32
{
    return sheen_LUT(vec2<f32>(NdotV, sheenRoughnessFactor)).r;
}

// Fresnel equations for dielectric/dielectric interfaces.
// Ref: https://belcour.github.io/blog/research/2017/05/01/brdf-thin-film.html
// Evaluation XYZ sensitivity curves in Fourier space
fn eval_sensitivity(OPD: f32, shift: vec3<f32>) -> vec3<f32> {
    let phase = 2.0 * MATH_PI * OPD * 1.0e-9;
    let val = vec3<f32>(5.4856e-13, 4.4201e-13, 5.2481e-13);
    let pos = vec3<f32>(1.6810e+06, 1.7953e+06, 2.2084e+06);
    let v = vec3<f32>(4.3278e+09, 9.3046e+09, 6.6121e+09);

    let p = phase*phase;
    var xyz = val * sqrt(2.0 * MATH_PI * v) * cos(pos * phase + shift) * exp(-p * v);
    xyz.x += 9.7470e-14 * sqrt(2.0 * MATH_PI * 4.5282e+09) * cos(2.2399e+06 * phase + shift[0]) * exp(-4.5282e+09 * p);
    xyz /= 1.0685e-7;

    let srgb = XYZ_TO_REC709 * xyz;
    return srgb;
}

fn eval_iridescence(outsideIOR: f32, eta2: f32, cosTheta1: f32, thinFilmThickness:f32, baseF0: vec3<f32>) -> vec3<f32> {
    var I = vec3<f32>(1.);

    // Force iridescenceIor -> outsideIOR when thinFilmThickness -> 0.0
    let iridescenceIor = mix(outsideIOR, eta2, smoothstep(0.0, 0.03, thinFilmThickness));
    // Evaluate the cosTheta on the base layer (Snell law)
    let ior = (outsideIOR / iridescenceIor);
    let sinTheta2Sq = ior * ior * (1.0 - (cosTheta1 * cosTheta1));

    // Handle TIR:
    let cosTheta2Sq = 1.0 - sinTheta2Sq;
    if (cosTheta2Sq < 0.0) {
        return I;
    }

    let cosTheta2 = sqrt(cosTheta2Sq);

    // First interface
    let R0 = ior_to_fresnel0_f32(iridescenceIor, outsideIOR);
    let R12 = f_schlick_f32(R0, 1.0, cosTheta1);
    let T121 = 1.0 - R12;
    var phi12 = 0.0;
    if (iridescenceIor < outsideIOR) { phi12 = MATH_PI; }
    let phi21 = MATH_PI - phi12;

    // Second interface
    let baseIOR = fresnel0_to_ior(clamp(baseF0, vec3<f32>(0.0), vec3<f32>(0.9999))); // guard against 1.0
    let R1 = ior_to_fresnel0_vec3(baseIOR, iridescenceIor);
    let R23 = f_schlick_vec3_f32(R1, 1.0, cosTheta2);
    var phi23 = vec3<f32>(0.0);
    if (baseIOR[0] < iridescenceIor) { phi23[0] = MATH_PI; }
    if (baseIOR[1] < iridescenceIor) { phi23[1] = MATH_PI; }
    if (baseIOR[2] < iridescenceIor) { phi23[2] = MATH_PI; }

    // Phase shift
    let OPD = 2.0 * iridescenceIor * thinFilmThickness * cosTheta2;
    let phi = vec3<f32>(phi21) + phi23;

    //// Compound terms
    let R123 = clamp(R12 * R23, vec3<f32>(1e-5), vec3<f32>(0.9999));
    let r123 = sqrt(R123);
    let Rs = (T121 * T121) * R23 / (vec3<f32>(1.0) - R123);

    //// Reflectance term for m = 0 (DC term amplitude)
    I = R12 + Rs;

    // Reflectance term for m > 0 (pairs of diracs)
    var Cm = Rs - T121;
    for (var m = 1; m <= 2; m++)
    {
        Cm *= r123;
        let Sm = 2.0 * eval_sensitivity(f32(m) * OPD, f32(m) * phi);
        I += Cm * Sm;
    }

    // Since out of gamut colors might be produced, negative color values are clamped to 0.
    return max(I, vec3<f32>(0.0));
}

fn BRDF_lambertian_iridescence(f0: vec3<f32>, f90: vec3<f32>, iridescenceFresnel: vec3<f32>, iridescenceFactor: f32, diffuseColor: vec3<f32>, specularWeight: f32, VdotH: f32) -> vec3<f32> {
    // Use the maximum component of the iridescence Fresnel color
    // Maximum is used instead of the RGB value to not get inverse colors for the diffuse BRDF
    let iridescenceFresnelMax = vec3<f32>(max(max(iridescenceFresnel.r, iridescenceFresnel.g), iridescenceFresnel.b));

    let schlickFresnel = f_schlick_vec3_vec3(f0, f90, VdotH);

    // Blend default specular Fresnel with iridescence Fresnel
    let F = mix(schlickFresnel, iridescenceFresnelMax, iridescenceFactor);

    // see https://seblagarde.wordpress.com/2012/01/08/pi-or-not-to-pi-in-game-lighting-equation/
    return (1.0 - specularWeight * F) * (diffuseColor / MATH_PI);
}

fn BRDF_specular_GGX_anisotropy(f0: vec3<f32>, f90: vec3<f32>, alphaRoughness: f32, anisotropy: f32, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, h: vec3<f32>, t: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    // Roughness along the anisotropy bitangent is the material roughness, while the tangent roughness increases with anisotropy.
    let at = mix(alphaRoughness, 1.0, anisotropy * anisotropy);
    let ab = clamp(alphaRoughness, 0.001, 1.0);

    let NdotL = clamp(dot(n, l), 0.0, 1.0);
    let NdotH = clamp(dot(n, h), 0.001, 1.0);
    let NdotV = dot(n, v);
    let VdotH = clamp(dot(v, h), 0.0, 1.0);

    let V = V_GGX_anisotropic(NdotL, NdotV, dot(b, v), dot(t, v), dot(t, l), dot(b, l), at, ab);
    let D = D_GGX_anisotropic(NdotH, dot(t, h), dot(b, h), anisotropy, at, ab);

    let F = f_schlick_vec3_vec3(f0, f90, VdotH);
    return F * V * D;
}

fn BRDF_specular_GGX_iridescence(f0: vec3<f32>, f90: vec3<f32>, iridescenceFresnel: vec3<f32>, alphaRoughness: f32, iridescenceFactor: f32, specularWeight: f32, VdotH: f32, NdotL: f32, NdotV: f32, NdotH: f32) -> vec3<f32> {
    let F = mix(f_schlick_vec3_vec3(f0, f90, VdotH), iridescenceFresnel, iridescenceFactor);
    let Vis = V_GGX(NdotL, NdotV, alphaRoughness);
    let D = D_GGX(NdotH, alphaRoughness);

    return specularWeight * F * Vis * D;
}

fn BRDF_lambertian(f0: vec3<f32>, f90: vec3<f32>, diffuseColor: vec3<f32>, specularWeight: f32, VdotH: f32) -> vec3<f32> {
    // see https://seblagarde.wordpress.com/2012/01/08/pi-or-not-to-pi-in-game-lighting-equation/
    return (1.0 - specularWeight * f_schlick_vec3_vec3(f0, f90, VdotH)) * (diffuseColor / MATH_PI);
}

fn BRDF_specular_GGX(f0: vec3<f32>, f90: vec3<f32>, alphaRoughness: f32, specularWeight: f32, VdotH: f32, NdotL: f32, NdotV: f32, NdotH: f32) -> vec3<f32> {
    let F = f_schlick_vec3_vec3(f0, f90, VdotH);
    let Vis = V_GGX(NdotL, NdotV, alphaRoughness);
    let D = D_GGX(NdotH, alphaRoughness);

    return specularWeight * F * Vis * D;
}

fn BRDF_specular_sheen(sheenColor: vec3<f32>, sheenRoughness: f32, NdotL: f32, NdotV: f32, NdotH: f32) -> vec3<f32> {
    let sheenDistribution = D_Charlie(sheenRoughness, NdotH);
    let sheenVisibility = V_Sheen(NdotL, NdotV, sheenRoughness);
    return sheenColor * sheenDistribution * sheenVisibility;
}

fn get_punctual_radiance_sheen(sheenColor: vec3<f32>, sheenRoughness: f32, NdotL: f32, NdotV: f32, NdotH: f32) -> vec3<f32> {
    return NdotL * BRDF_specular_sheen(sheenColor, sheenRoughness, NdotL, NdotV, NdotH);
}

fn get_punctual_radiance_clearcoat(clearcoatNormal: vec3<f32>, v: vec3<f32>, l: vec3<f32>, h: vec3<f32>, VdotH: f32, f0: vec3<f32>, f90: vec3<f32>, clearcoatRoughness: f32) -> vec3<f32> {
    let NdotL = clamped_dot(clearcoatNormal, l);
    let NdotV = clamped_dot(clearcoatNormal, v);
    let NdotH = clamped_dot(clearcoatNormal, h);
    return NdotL * BRDF_specular_GGX(f0, f90, clearcoatRoughness * clearcoatRoughness, 1.0, VdotH, NdotL, NdotV, NdotH);
}

fn get_punctual_radiance_transmission(normal: vec3<f32>, view: vec3<f32>, pointToLight: vec3<f32>, alphaRoughness: f32, f0: vec3<f32>, f90: vec3<f32>, baseColor: vec3<f32>, ior: f32) -> vec3<f32> {
    let transmissionRougness = apply_ior_to_roughness(alphaRoughness, ior);

    let n = normalize(normal);           // Outward direction of surface point
    let v = normalize(view);             // Direction from surface point to view
    let l = normalize(pointToLight);
    let l_mirror = normalize(l + 2.0*n*dot(-l, n));     // Mirror light reflection vector on surface
    let h = normalize(l_mirror + v);            // Halfway vector between transmission light vector and v

    let D = D_GGX(clamp(dot(n, h), 0.0, 1.0), transmissionRougness);
    let F = f_schlick_vec3_vec3(f0, f90, clamp(dot(v, h), 0.0, 1.0));
    let Vis = V_GGX(clamp(dot(n, l_mirror), 0.0, 1.0), clamp(dot(n, v), 0.0, 1.0), transmissionRougness);

    // Transmission BTDF
    return (1.0 - F) * baseColor * D * Vis;
}

fn get_range_attenuation(range: f32, distance: f32) -> f32 {
    if (range <= 0.0)
    {
        // negative range means unlimited
        return 1.0 / pow(distance, 2.0);
    }
    return max(min(1.0 - pow(distance / range, 4.0), 1.0), 0.0) / pow(distance, 2.0);
}

fn get_spot_attenuation(pointToLight: vec3<f32>, spotDirection: vec3<f32>, outerConeCos: f32, innerConeCos: f32) -> f32 {
    let actualCos = dot(normalize(spotDirection), normalize(-pointToLight));
    if (actualCos > outerConeCos)
    {
        if (actualCos < innerConeCos)
        {
            let angularAttenuation = (actualCos - outerConeCos) / (innerConeCos - outerConeCos);
            return angularAttenuation * angularAttenuation;
        }
        return 1.0;
    }
    return 0.0;
}

fn get_light_intensity(light: ptr<function, LightData>, pointToLight: vec3<f32>) -> vec3<f32> {
    var rangeAttenuation = 1.0;
    var spotAttenuation = 1.0;

    if ((*light).light_type != LIGHT_TYPE_DIRECTIONAL)
    {
        rangeAttenuation = get_range_attenuation((*light).range, length(pointToLight));
    }
    if ((*light).light_type == LIGHT_TYPE_SPOT)
    {
        spotAttenuation = get_spot_attenuation(pointToLight, (*light).direction, (*light).outer_cone_angle, (*light).inner_cone_angle);
    }
    return rangeAttenuation * spotAttenuation * (*light).intensity * (*light).color;
}

fn get_volume_transmission_ray(n: vec3<f32>, v: vec3<f32>, thickness: f32, ior: f32, scale: f32) -> vec3<f32> {
    // Direction of refracted light.
    let refractionVector = refract(-v, normalize(n), 1.0 / ior);
    // The thickness is specified in local space.
    return normalize(refractionVector) * thickness * scale;
}

// Compute attenuated light as it travels through a volume.
fn apply_volume_attenuation(radiance: vec3<f32>, transmissionDistance: f32, attenuationColor: vec3<f32>, attenuationDistance: f32) -> vec3<f32> {
    if (attenuationDistance == 0.0)
    {
        // Attenuation distance is +∞ (which we indicate by zero), i.e. the transmitted color is not attenuated at all.
        return radiance;
    }
    else
    {
        // Compute light attenuation using Beer's law.
        let attenuationCoefficient = -log(attenuationColor) / attenuationDistance;
        let transmittance = exp(-attenuationCoefficient * transmissionDistance); // Beer's law
        return transmittance * radiance;
    }
}
/*
fn get_IBL_radiance_GGX_iridescence(n: vec3<f32>, v: vec3<f32>, roughness: f32, F0: vec3<f32>, iridescenceFresnel: vec3<f32>, iridescenceFactor: f32, specularWeight: f32) -> vec3<f32> {
	let NdotV: f32 = clampedDot(n, v);
	let lod: f32 = roughness * f32(u_MipCount - 1.);
	let reflection: vec3<f32> = normalize(reflect(-v, n));
	let brdfSamplePoint: vec2<f32> = clamp(vec2<f32>(NdotV, roughness), vec2<f32>(0., 0.), vec2<f32>(1., 1.));
	let f_ab: vec2<f32> = sample_texture(BUFFER_u_GGXLUT, brdfSamplePoint).rg;
	let specularSample: vec4<f32> = getSpecularSample(reflection, lod);
	let specularLight: vec3<f32> = specularSample.rgb;
	let Fr: vec3<f32> = max(vec3<f32>(1. - roughness), F0) - F0;
	let k_S: vec3<f32> = mix(F0 + Fr * pow(1. - NdotV, 5.), iridescenceFresnel, iridescenceFactor);
	let FssEss: vec3<f32> = k_S * f_ab.x + f_ab.y;
	return specularWeight * specularLight * FssEss;
} 
*/
//Inspired from glTF-Sample-Viewer
fn compute_color_from_material(material_id: u32, pixel_data: ptr<function, PixelData>) -> MaterialInfo {
    var material = materials.data[material_id];

    let v = normalize(matrix_row(constant_data.view, 2u).xyz - (*pixel_data).world_pos);
    var tbn = compute_tbn(&material, pixel_data);

    let NdotV = clamped_dot(tbn.normal,v);
    let TdotV = clamped_dot(tbn.tangent,v);
    let BdotV = clamped_dot(tbn.binormal,v);

    var material_info: MaterialInfo;

    init_material_info_default(&material_info);
    compute_base_color(&material, pixel_data, &material_info);

    if((material.flags & MATERIAL_FLAGS_IOR) != 0u) {
        compute_ior(&material, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_SPECULARGLOSSINESS) != 0u) {
        compute_specular_glossiness(&material, pixel_data, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_METALLICROUGHNESS) != 0u) {
        compute_metallic_roughness(&material, pixel_data, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_SHEEN) != 0u) {
        compute_sheen(&material, pixel_data, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_CLEARCOAT) != 0u) {
        compute_clear_coat(&material, tbn.normal, pixel_data, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_SPECULAR) != 0u) {
        compute_specular(&material, pixel_data, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_TRANSMISSION) != 0u) {
        compute_transmission(&material, pixel_data, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_VOLUME) != 0u) {
        compute_volume(&material, pixel_data, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_IRIDESCENCE) != 0u) {
        compute_iridescence(&material, pixel_data, &material_info);
    }
    if((material.flags & MATERIAL_FLAGS_ANISOTROPY) != 0u) {
        compute_anisotropy(&material, &tbn, &material_info);
    }

    material_info.perceptual_roughness = clamp(material_info.perceptual_roughness, 0.0, 1.0);
    material_info.metallic = clamp(material_info.metallic, 0.0, 1.0);
    
    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness.
    material_info.alpha_roughness = material_info.perceptual_roughness * material_info.perceptual_roughness;

    // Compute reflectance.
    let reflectance = max(max(material_info.f0.r, material_info.f0.g), material_info.f0.b);
    
    // Anything less than 2% is physically impossible and is instead considered to be shadowing. Compare to "Real-Time-Rendering" 4th editon on page 325.
    material_info.f90 = vec3<f32>(1.);

    // LIGHTING
    var f_specular = vec3<f32>(0.0);
    var f_diffuse = vec3<f32>(0.0);
    var f_emissive = vec3<f32>(0.0);
    var f_clearcoat = vec3<f32>(0.0);
    var f_sheen = vec3<f32>(0.0);
    var f_transmission = vec3<f32>(0.0);

    var albedo_sheen_scaling = 1.0;

    var iridescence_fresnel = vec3<f32>(0.);
    var iridescence_f0 = vec3<f32>(0.);
    if((material.flags & MATERIAL_FLAGS_IRIDESCENCE) != 0u) {
        iridescence_fresnel = eval_iridescence(1., material_info.iridescence_ior, NdotV, material_info.iridescence_thickness, material_info.f0);
        iridescence_f0 = schlick_to_f0_vec3(iridescence_fresnel, vec3<f32>(1.), NdotV);
        if(material_info.iridescence_thickness == 0.) { material_info.iridescence_factor = 0.; }
    }

    let specular_weight_and_anisotropy_strength = unpack2x16float(material_info.specular_weight_and_anisotropy_strength);
    let specular_weight = specular_weight_and_anisotropy_strength.x;
    let anisotropy_strength = specular_weight_and_anisotropy_strength.y;

/*
    // TODO - Calculate lighting contribution from image based lighting source (IBL)
    if ((constant_data.flags & CONSTANT_DATA_FLAGS_USE_IBL) != 0) {      
        if((material.flags & MATERIAL_FLAGS_IRIDESCENCE) != 0u) {
            f_specular += get_IBL_radiance_GGX_iridescence(tbn.normal, v, material_info.perceptual_roughness, material_info.f0, iridescence_fresnel, material_info.iridescence_factor, specular_weight);
            f_diffuse += get_IBL_radiance_lambertian_iridescence(tbn.normal, v, material_info.perceptual_roughness, material_info.c_diff, material_info.f0, iridescence_f0, material_info.iridescence_factor, specular_weight);
        }
        else if((material.flags & MATERIAL_FLAGS_ANISOTROPY) != 0u) {
            f_specular += get_IBL_radiance_anisotropy(tbn.normal, v, material_info.perceptual_roughness, material_info.anisotropy_strength, material_info.anisotropicB, material_info.f0, specular_weight);
            f_diffuse += get_IBL_radiance_lambertian(tbn.normal, v, material_info.perceptual_roughness, material_info.c_diff, material_info.f0, specular_weight);
        }
        else {
            f_specular += get_IBL_radiance_GGX(tbn.normal, v, material_info.perceptual_roughness, material_info.f0, specular_weight);
            f_diffuse += get_IBL_radiance_lambertian(tbn.normal, v, material_info.perceptual_roughness, material_info.c_diff, material_info.f0, specular_weight);
        }
        if((material.flags & MATERIAL_FLAGS_CLEARCOAT) != 0u) {
            f_clearcoat += get_IBL_radiance_GGX(material_info.clear_coat_normal, v, material_info.clear_coat_roughness_factor, material_info.clear_coat_f0, 1.0);
        }
        if((material.flags & MATERIAL_FLAGS_SHEEN) != 0u) {
            f_sheen += get_IBL_radiance_charlie(tbn.normal, v, material_info.sheen_color_and_roughness_factor.z, material_info.sheen_color_and_roughness_factor.rgb);
        }
        if((material.flags & MATERIAL_FLAGS_TRANSMISSION) != 0u) {
            let mesh = &meshes.data[(*pixel_data).mesh_id];
            let mesh_matrix = transform_matrix((*mesh).position, (*mesh).orientation, (*mesh).scale);
            f_transmission += get_IBL_volume_refraction(
                tbn.normal, v,
                material_info.perceptual_roughness,
                material_info.c_diff, material_info.f0, material_info.f90,
                (*pixel_data.world_pos), mesh_matrix, constant_data.view, constant_data.proj,
                material_info.ior, material_info.thickness_factor, material_info.attenuation_color_and_distance.rgb, material_info.attenuation_color_and_distance.z);
        }
    }
*/  
    var f_diffuse_ibl = f_diffuse;
    var f_specular_ibl = f_specular;
    var f_sheen_ibl = f_sheen;
    var f_clearcoat_ibl = f_clearcoat;
    f_diffuse = material_info.base_color.rgb;
    f_specular = vec3<f32>(0.0);
    f_sheen = vec3<f32>(0.0);
    f_clearcoat = vec3<f32>(0.0);
   
    let num_lights = arrayLength(&lights.data);
    let i = hash(constant_data.frame_index) % num_lights;
    
    var light = lights.data[i];
    if (light.light_type != LIGHT_TYPE_INVALID) { 
        f_diffuse = vec3<f32>(0.0);
        var pointToLight: vec3<f32>;
        if (light.light_type != LIGHT_TYPE_DIRECTIONAL) { 
            pointToLight = light.position - (*pixel_data).world_pos;
        } else {
            pointToLight = -light.direction;
        }
            
        // BSTF
        var l = normalize(pointToLight);   // Direction from surface point to light
        let h = normalize(l + v);          // Direction of the vector between l and v, called halfway vector
        let NdotL = clamped_dot(tbn.normal, l);
        let NdotH = clamped_dot(tbn.normal, h);
        let NdotV = clamped_dot(tbn.normal, v);
        let LdotH = clamped_dot(l, h);
        let VdotH = clamped_dot(v, h);
        if (NdotL > 0.0 || NdotV > 0.0)
        {
            // Calculation of analytical light
            let intensity = get_light_intensity(&light, pointToLight);
            if((material.flags & MATERIAL_FLAGS_IRIDESCENCE) != 0u) {
                f_diffuse += intensity * NdotL *  BRDF_lambertian_iridescence(material_info.f0, material_info.f90, iridescence_fresnel, material_info.iridescence_factor, material_info.c_diff, specular_weight, VdotH);
                f_specular += intensity * NdotL * BRDF_specular_GGX_iridescence(material_info.f0, material_info.f90, iridescence_fresnel, material_info.alpha_roughness,  material_info.iridescence_factor, specular_weight, VdotH, NdotL, NdotV, NdotH);
            }
            else if((material.flags & MATERIAL_FLAGS_ANISOTROPY) != 0u) {
                f_diffuse += intensity * NdotL *  BRDF_lambertian(material_info.f0, material_info.f90, material_info.c_diff, specular_weight, VdotH);
                f_specular += intensity * NdotL * BRDF_specular_GGX_anisotropy(material_info.f0, material_info.f90, material_info.alpha_roughness, anisotropy_strength, tbn.normal, v, l, h, material_info.anisotropicT, material_info.anisotropicB);
            } else {
                f_diffuse += intensity * NdotL *  BRDF_lambertian(material_info.f0, material_info.f90, material_info.c_diff, specular_weight, VdotH);
                f_specular += intensity * NdotL * BRDF_specular_GGX(material_info.f0, material_info.f90, material_info.alpha_roughness, specular_weight, VdotH, NdotL, NdotV, NdotH);
            }
            
            if((material.flags & MATERIAL_FLAGS_SHEEN) != 0u) {
                f_sheen += intensity * get_punctual_radiance_sheen(material_info.sheen_color_and_roughness_factor.rgb, material_info.sheen_color_and_roughness_factor.w, NdotL, NdotV, NdotH);
                let c = max(max(material_info.sheen_color_and_roughness_factor.r, material_info.sheen_color_and_roughness_factor.g), material_info.sheen_color_and_roughness_factor.b);
                albedo_sheen_scaling = min(1.0 - c * albedo_sheen_scaling_LUT(NdotV, material_info.sheen_color_and_roughness_factor.w),
                    1.0 - c * albedo_sheen_scaling_LUT(NdotL, material_info.sheen_color_and_roughness_factor.w));
            }
            
            if((material.flags & MATERIAL_FLAGS_CLEARCOAT) != 0u) {
                f_clearcoat += intensity * get_punctual_radiance_clearcoat(material_info.clear_coat_normal, v, l, h, VdotH,
                    material_info.clear_coat_f0, material_info.clear_coat_f90, material_info.clear_coat_roughness_factor);
            }
        }
        
        // BDTF
        if((material.flags & MATERIAL_FLAGS_TRANSMISSION) != 0u) {
            // If the light ray travels through the geometry, use the point it exits the geometry again.
            // That will change the angle to the light source, if the material refracts the light ray.
            let mesh = &meshes.data[(*pixel_data).mesh_id];
            let mesh_scale = (*mesh).scale.x;
            let transmission_ray = get_volume_transmission_ray(tbn.normal, v, material_info.thickness_factor, material_info.ior, mesh_scale);
            pointToLight -= transmission_ray;
            l = normalize(pointToLight);
            let intensity = get_light_intensity(&light, pointToLight);
            var transmitted_light = intensity * get_punctual_radiance_transmission(tbn.normal, v, l, material_info.alpha_roughness, material_info.f0, material_info.f90, material_info.c_diff, material_info.ior);
        
            if((material.flags & MATERIAL_FLAGS_VOLUME) != 0u) {
                transmitted_light = apply_volume_attenuation(transmitted_light, length(transmission_ray), material_info.attenuation_color_and_distance.rgb, material_info.attenuation_color_and_distance.w);
            }
            f_transmission += transmitted_light;
        }
    }
    
    f_emissive = material.emissive_color.rgb * material.emissive_strength;
    if (has_texture(&material, TEXTURE_TYPE_EMISSIVE)) {  
      let uv = material_texture_uv(&material, pixel_data, TEXTURE_TYPE_EMISSIVE);
      let texture_color = sample_texture(uv);
      f_emissive *= texture_color.rgb;
    }

    //Layer blending

    var clearcoatFactor = 0.0;
    var clearcoatFresnel = vec3(0.);
    var diffuse = vec3<f32>(0.0);
    var specular = vec3<f32>(0.0);
    var sheen = vec3<f32>(0.0);
    var clearcoat = vec3<f32>(0.0);
   
    var ao = 1.0;
    if (has_texture(&material, TEXTURE_TYPE_OCCLUSION)) {  
        let uv = material_texture_uv(&material, pixel_data, TEXTURE_TYPE_OCCLUSION);
        let texture_color = sample_texture(uv);
        ao = ao * texture_color.r;
        diffuse = f_diffuse + mix(f_diffuse_ibl, f_diffuse_ibl * ao, material.occlusion_strength);
        // apply ambient occlusion to all lighting that is not punctual
        specular = f_specular + mix(f_specular_ibl, f_specular_ibl * ao, material.occlusion_strength);
        sheen = f_sheen + mix(f_sheen_ibl, f_sheen_ibl * ao, material.occlusion_strength);
        clearcoat = f_clearcoat + mix(f_clearcoat_ibl, f_clearcoat_ibl * ao, material.occlusion_strength);
    }
    else {
        diffuse = f_diffuse_ibl + f_diffuse;
        specular = f_specular_ibl + f_specular;
        sheen = f_sheen_ibl + f_sheen;
        clearcoat = f_clearcoat_ibl + f_clearcoat;
    }
    
    if((material.flags & MATERIAL_FLAGS_CLEARCOAT) != 0u) {
        clearcoatFactor = material_info.clear_coat_factor;
        clearcoatFresnel = f_schlick_vec3_vec3(material_info.clear_coat_f0, material_info.clear_coat_f90, clamped_dot(material_info.clear_coat_normal, v));
        clearcoat *= clearcoatFactor;
    }
    
    if((material.flags & MATERIAL_FLAGS_TRANSMISSION) != 0u) {
        diffuse = mix(diffuse, f_transmission, material_info.transmission_factor);
    }

    var color = vec3<f32>(0.);
    if((material.flags & MATERIAL_FLAGS_UNLIT) != 0u) {
        color = material_info.base_color.rgb;
    } else {
        color = f_emissive + diffuse + specular;
        color = sheen + color * albedo_sheen_scaling;
        color = color * (1.0 - clearcoatFactor * clearcoatFresnel) + clearcoat;
    }

    material_info.f_color = vec4<f32>(color, material_info.base_color.a);
    material_info.f_emissive = f_emissive;
    material_info.f_diffuse = f_diffuse;
    material_info.f_diffuse_ibl = f_diffuse_ibl;
    material_info.f_specular = f_specular;
    return material_info;
}