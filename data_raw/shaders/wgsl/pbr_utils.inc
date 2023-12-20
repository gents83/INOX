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

fn f_schlick_f32(f0: f32, f90: f32, VdotH: f32) -> f32 {
    let x = clamp(1.0 - VdotH, 0.0, 1.0);
    let x2 = x * x;
    let x5 = x * x2 * x2;
    return f0 + (f90 - f0) * x5;
}

fn f_schlick_vec3(f0: vec3<f32>, f90: f32, VdotH: f32) -> vec3<f32>
{
    let x = clamp(1.0 - VdotH, 0.0, 1.0);
    let x2 = x * x;
    let x5 = x * x2 * x2;
    return f0 + (f90 - f0) * x5;
}

fn schlick_to_f0_vec3(f: vec3<f32>, f90: vec3<f32>, VdotH: f32) -> vec3<f32>{
    let x = clamp(1.0 - VdotH, 0.0, 1.0);
    let x2 = x * x;
    let x5 = clamp(x * x2 * x2, 0.0, 0.9999);
    return (f - f90 * x5) / (1.0 - x5);
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
    let R23 = f_schlick_vec3(R1, 1.0, cosTheta2);
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
    return max(I, vec3(0.0));
}

//Inspired from glTF-Sample-Viewer
fn compute_color_from_material(material_id: u32, pixel_data: ptr<function, PixelData>) -> MaterialInfo {
    var material = materials.data[material_id];

    let v = normalize(constant_data.view[3].xyz - (*pixel_data).world_pos);
    let tbn = compute_tbn(&material, pixel_data);

    let NdotV = clamped_dot(tbn.normal,v);
    let TdotV = clamped_dot(tbn.tangent,v);
    let BdotV = clamped_dot(tbn.binormal,v);

    let ior = 1.5;
    let f0 = vec3<f32>(0.04);
    let specular_weight = 1.0;

    var material_info: MaterialInfo;
    compute_base_color(&material, pixel_data, &material_info);
    compute_ior(&material, &material_info);
    compute_specular_glossiness(&material, pixel_data, &material_info);
    compute_metallic_roughness(&material, pixel_data, &material_info);
    compute_sheen(&material, pixel_data, &material_info);
    compute_clear_coat(&material, pixel_data, &material_info);
    compute_specular(&material, pixel_data, &material_info);
    compute_transmission(&material, pixel_data, &material_info);
    compute_volume(&material, pixel_data, &material_info);
    compute_iridescence(&material, pixel_data, &material_info);
    compute_anisotropy(&material, pixel_data, &material_info);

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
    let f_specular = vec3<f32>(0.0);
    let f_diffuse = vec3<f32>(0.0);
    let f_emissive = vec3<f32>(0.0);
    let f_clearcoat = vec3<f32>(0.0);
    let f_sheen = vec3<f32>(0.0);
    let f_transmission = vec3<f32>(0.0);

    let albedo_sheen_scaling = 1.0;

    let iridescence_fresnel = eval_iridescence(1., material_info.iridescence_ior, NdotV, material_info.iridescence_thickness, material_info.f0);
    let iridescence_f0 = schlick_to_f0_vec3(iridescence_fresnel, vec3<f32>(1.0), NdotV);
    if(material_info.iridescence_thickness == 0.) { material_info.iridescence_factor = 0.; }

    //TODO - here we should fill specular and diffuse with lighting contribution calculated from image based lighting source (IBL)


    material_info.emissive_color = material.emissive_color;
    return material_info;
}