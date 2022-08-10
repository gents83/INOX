let MAX_TEXTURE_ATLAS_COUNT: u32 = 8u;
let MAX_NUM_LIGHTS: u32 = 64u;
let MAX_NUM_TEXTURES: u32 = 512u;
let MAX_NUM_MATERIALS: u32 = 512u;

let TEXTURE_TYPE_BASE_COLOR: u32 = 0u;
let TEXTURE_TYPE_METALLIC_ROUGHNESS: u32 = 1u;
let TEXTURE_TYPE_NORMAL: u32 = 2u;
let TEXTURE_TYPE_EMISSIVE: u32 = 3u;
let TEXTURE_TYPE_OCCLUSION: u32 = 4u;
let TEXTURE_TYPE_SPECULAR_GLOSSINESS: u32 = 5u;
let TEXTURE_TYPE_DIFFUSE: u32 = 6u;
let TEXTURE_TYPE_EMPTY_FOR_PADDING: u32 = 7u;
let TEXTURE_TYPE_COUNT: u32 = 8u;

let MATERIAL_ALPHA_BLEND_OPAQUE = 0u;
let MATERIAL_ALPHA_BLEND_MASK = 1u;
let MATERIAL_ALPHA_BLEND_BLEND = 2u;

let CONSTANT_DATA_FLAGS_NONE: u32 = 0u;
let CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;

struct ConstantData {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    screen_width: f32,
    screen_height: f32,
    flags: u32,
};

struct LightData {
    position: vec3<f32>,
    light_type: u32,
    color: vec4<f32>,
    intensity: f32,
    range: f32,
    inner_cone_angle: f32,
    outer_cone_angle: f32,
};

struct TextureData {
    texture_index: u32,
    layer_index: u32,
    total_width: f32,
    total_height: f32,
    area: vec4<f32>,
};

struct ShaderMaterialData {
    textures_indices: array<i32, 8>,//TEXTURE_TYPE_COUNT>,
    textures_coord_set: array<u32, 8>,//TEXTURE_TYPE_COUNT>,
    roughness_factor: f32,
    metallic_factor: f32,
    alpha_cutoff: f32,
    alpha_mode: u32,
    base_color: vec4<f32>,
    emissive_color: vec3<f32>,
    occlusion_strength: f32,
    diffuse_color: vec4<f32>,
    specular_color: vec4<f32>,
};

struct DynamicData {
    textures_data: array<TextureData, 512>,//MAX_NUM_TEXTURES>,
    materials_data: array<ShaderMaterialData, 512>,//MAX_NUM_MATERIALS>,
    lights_data: array<LightData, 64>,//MAX_NUM_LIGHTS>,
};


struct VertexInput {
    //@builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) tex_coords_0: vec2<f32>,
    @location(5) tex_coords_1: vec2<f32>,
    @location(6) tex_coords_2: vec2<f32>,
    @location(7) tex_coords_3: vec2<f32>,
};

struct InstanceInput {
    //@builtin(instance_index) index: u32,
    @location(8) draw_area: vec4<f32>,
    @location(9) model_matrix_0: vec4<f32>,
    @location(10) model_matrix_1: vec4<f32>,
    @location(11) model_matrix_2: vec4<f32>,
    @location(12) model_matrix_3: vec4<f32>,
    @location(13) material_index: i32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    //@builtin(front_facing) is_front_facing: bool,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_tangent: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) @interpolate(flat) material_index: i32,
    @location(5) tex_coords_base_color: vec2<f32>,
    @location(6) tex_coords_metallic_roughness: vec2<f32>,
    @location(7) tex_coords_normal: vec2<f32>,
    @location(8) tex_coords_emissive: vec2<f32>,
    @location(9) tex_coords_occlusion: vec2<f32>,
    @location(10) tex_coords_specular_glossiness: vec2<f32>,
    @location(11) tex_coords_diffuse: vec2<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> dynamic_data: DynamicData;

@group(1) @binding(0)
var default_sampler: sampler;
@group(1) @binding(1)
var depth_sampler: sampler_comparison;

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
@group(1) @binding(2)
var texture_array: binding_array<texture_2d<f32>, 16>; //MAX_TEXTURE_ATLAS_COUNT
#else
@group(1) @binding(2)
var texture_1: texture_2d<f32>;
@group(1) @binding(3)
var texture_2: texture_2d<f32>;
@group(1) @binding(4)
var texture_3: texture_2d<f32>;
@group(1) @binding(5)
var texture_4: texture_2d<f32>;
@group(1) @binding(6)
var texture_5: texture_2d<f32>;
@group(1) @binding(7)
var texture_6: texture_2d<f32>;
@group(1) @binding(8)
var texture_7: texture_2d<f32>;
@group(1) @binding(9)
var texture_8: texture_2d<f32>;
#endif

fn inverse_transpose_3x3(m: mat3x3<f32>) -> mat3x3<f32> {
    let x = cross(m[1], m[2]);
    let y = cross(m[2], m[0]);
    let z = cross(m[0], m[1]);
    let det = dot(m[2], z);
    return mat3x3<f32>(
        x / det,
        y / det,
        z / det
    );
}

fn get_textures_coord_set(v: VertexInput, material_index: i32, texture_type: u32) -> vec2<f32> {
    let textures_coord_set_index = dynamic_data.materials_data[material_index].textures_coord_set[texture_type];
    if (textures_coord_set_index == 1u) {
        return v.tex_coords_1;
    } else if (textures_coord_set_index == 2u) {
        return v.tex_coords_2;
    } else if (textures_coord_set_index == 3u) {
        return v.tex_coords_3;
    }
    return v.tex_coords_0;
}


@vertex
fn vs_main(
    v: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let instance_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.model_matrix_0.xyz,
        instance.model_matrix_1.xyz,
        instance.model_matrix_2.xyz,
    );
    let inv_normal_matrix = inverse_transpose_3x3(normal_matrix);

    var vertex_out: VertexOutput;
    vertex_out.world_position = instance_matrix * vec4<f32>(v.position, 1.0);
    vertex_out.clip_position = constant_data.proj * constant_data.view * vertex_out.world_position;
    vertex_out.world_normal = inv_normal_matrix * v.normal;
    vertex_out.world_tangent = vec4<f32>(normal_matrix * v.tangent.xyz, v.tangent.w);
    vertex_out.color = v.color;
    vertex_out.material_index = instance.material_index;

    if (instance.material_index >= 0) {
        vertex_out.tex_coords_base_color = get_textures_coord_set(v, instance.material_index, TEXTURE_TYPE_BASE_COLOR);
        vertex_out.tex_coords_metallic_roughness = get_textures_coord_set(v, instance.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS);
        vertex_out.tex_coords_normal = get_textures_coord_set(v, instance.material_index, TEXTURE_TYPE_NORMAL);
        vertex_out.tex_coords_emissive = get_textures_coord_set(v, instance.material_index, TEXTURE_TYPE_EMISSIVE);
        vertex_out.tex_coords_occlusion = get_textures_coord_set(v, instance.material_index, TEXTURE_TYPE_OCCLUSION);
        vertex_out.tex_coords_specular_glossiness = get_textures_coord_set(v, instance.material_index, TEXTURE_TYPE_SPECULAR_GLOSSINESS);
        vertex_out.tex_coords_diffuse = get_textures_coord_set(v, instance.material_index, TEXTURE_TYPE_DIFFUSE);
    }

    return vertex_out;
}

fn has_texture(material_index: i32, texture_type: u32) -> bool {
    if (material_index < 0) {
        return false;
    }
    if (dynamic_data.materials_data[u32(material_index)].textures_indices[texture_type] >= 0) {
        return true;
    }
    return false;
}

fn wrap(a: f32, min: f32, max: f32) -> f32 {
    if (a < min) {
        return max - (min - a);
    }
    if (a > max) {
        return min + (a - max);
    }
    return a;
}

fn get_texture_color(material_index: i32, texture_type: u32, tex_coords: vec2<f32>) -> vec4<f32> {
    if (material_index < 0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let texture_data_index = dynamic_data.materials_data[material_index].textures_indices[texture_type];
    var t = vec3<f32>(0.0, 0.0, 0.0);
    let area = dynamic_data.textures_data[texture_data_index].area;
    let image_width = dynamic_data.textures_data[texture_data_index].total_width;
    let image_height = dynamic_data.textures_data[texture_data_index].total_height;
    let fract_x = min(fract(tex_coords.x), 1.0);
    let fract_y = min(fract(tex_coords.y), 1.0);
    t.x = (area.x + 0.5 + area.z * fract_x) / image_width;
    t.y = (area.y + 0.5 + area.w * fract_y) / image_height;
    t.z = f32(dynamic_data.textures_data[texture_data_index].layer_index);

    let atlas_index = dynamic_data.textures_data[texture_data_index].texture_index;
#ifdef FEATURES_TEXTURE_BINDING_ARRAY
    return textureSampleLevel(texture_array[atlas_index], default_sampler, t.xy, t.z);
#else
    if (atlas_index == 1u) {
        return textureSampleLevel(texture_2, default_sampler, t.xy, t.z);
    } else if (atlas_index == 2u) {
        return textureSampleLevel(texture_3, default_sampler, t.xy, t.z);
    } else if (atlas_index == 3u) {
        return textureSampleLevel(texture_4, default_sampler, t.xy, t.z);
    } else if (atlas_index == 4u) {
        return textureSampleLevel(texture_5, default_sampler, t.xy, t.z);
    } else if (atlas_index == 5u) {
        return textureSampleLevel(texture_6, default_sampler, t.xy, t.z);
    } else if (atlas_index == 6u) {
        return textureSampleLevel(texture_7, default_sampler, t.xy, t.z);
    } else if (atlas_index == 7u) {
        return textureSampleLevel(texture_8, default_sampler, t.xy, t.z);
    } else if (atlas_index == 8u) {
        return textureSampleLevel(texture_9, default_sampler, t.xy, t.z);
    }
    return textureSampleLevel(texture_1, default_sampler, t.xy, t.z);
#endif
}

let PI: f32 = 3.141592653589793;
let AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.95, 0.95, 0.95);
let SHADOW_DEPTH_BIAS: f32 = 0.02;
let SHADOW_NORMAL_BIAS: f32 = 0.6;

// From https://www.unrealengine.com/en-US/blog/physically-based-shading-on-mobile
fn EnvBRDFApprox(f0: vec3<f32>, perceptual_roughness: f32, NoV: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = perceptual_roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * NoV)) * r.x + r.y;
    let AB = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    return f0 * AB.x + AB.y;
}

fn perceptualRoughnessToRoughness(perceptualRoughness: f32) -> f32 {
    // clamp perceptual roughness to prevent precision problems
    // According to Filament design 0.089 is recommended for mobile
    // Filament uses 0.045 for non-mobile
    let clampedPerceptualRoughness = clamp(perceptualRoughness, 0.089, 1.0);
    return clampedPerceptualRoughness * clampedPerceptualRoughness;
}

// luminance coefficients from Rec. 709.
// https://en.wikipedia.org/wiki/Rec._709
fn luminance(v: vec3<f32>) -> f32 {
    return dot(v, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn change_luminance(c_in: vec3<f32>, l_out: f32) -> vec3<f32> {
    let l_in = luminance(c_in);
    return c_in * (l_out / l_in);
}

fn reinhard_luminance(color: vec3<f32>) -> vec3<f32> {
    let l_old = luminance(color);
    let l_new = l_old / (1.0 + l_old);
    return change_luminance(color, l_new);
}

fn saturate(value: f32) -> f32 {
    return clamp(value, 0.0, 1.0);
}

// distanceAttenuation is simply the square falloff of light intensity
// combined with a smooth attenuation at the edge of the light radius
//
// light radius is a non-physical construct for efficiency purposes,
// because otherwise every light affects every fragment in the scene
fn getDistanceAttenuation(distanceSquare: f32, inverseRangeSquared: f32) -> f32 {
    let factor = distanceSquare * inverseRangeSquared;
    let smoothFactor = saturate(1.0 - factor * factor);
    let attenuation = smoothFactor * smoothFactor;
    return attenuation * 1.0 / max(distanceSquare, 0.0001);
}

// Normal distribution function (specular D)
// Based on https://google.github.io/filament/Filament.html#citation-walter07

// D_GGX(h,α) = α^2 / { π ((n⋅h)^2 (α2−1) + 1)^2 }

// Simple implementation, has precision problems when using fp16 instead of fp32
// see https://google.github.io/filament/Filament.html#listing_speculardfp16
fn D_GGX(roughness: f32, NoH: f32, h: vec3<f32>) -> f32 {
    let oneMinusNoHSquared = 1.0 - NoH * NoH;
    let a = NoH * roughness;
    let k = roughness / (oneMinusNoHSquared + a * a);
    let d = k * k * (1.0 / PI);
    return d;
}

// Visibility function (Specular G)
// V(v,l,a) = G(v,l,α) / { 4 (n⋅v) (n⋅l) }
// such that f_r becomes
// f_r(v,l) = D(h,α) V(v,l,α) F(v,h,f0)
// where
// V(v,l,α) = 0.5 / { n⋅l sqrt((n⋅v)^2 (1−α2) + α2) + n⋅v sqrt((n⋅l)^2 (1−α2) + α2) }
// Note the two sqrt's, that may be slow on mobile, see https://google.github.io/filament/Filament.html#listing_approximatedspecularv
fn V_SmithGGXCorrelated(roughness: f32, NoV: f32, NoL: f32) -> f32 {
    let a2 = roughness * roughness;
    let lambdaV = NoL * sqrt((NoV - a2 * NoV) * NoV + a2);
    let lambdaL = NoV * sqrt((NoL - a2 * NoL) * NoL + a2);
    let v = 0.5 / (lambdaV + lambdaL);
    return v;
}

// Fresnel function
// see https://google.github.io/filament/Filament.html#citation-schlick94
// F_Schlick(v,h,f_0,f_90) = f_0 + (f_90 − f_0) (1 − v⋅h)^5
fn F_Schlick_vec(f0: vec3<f32>, f90: f32, VoH: f32) -> vec3<f32> {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow(1.0 - VoH, 5.0);
}

fn F_Schlick(f0: f32, f90: f32, VoH: f32) -> f32 {
    // not using mix to keep the vec3 and float versions identical
    return f0 + (f90 - f0) * pow(1.0 - VoH, 5.0);
}

fn fresnel(f0: vec3<f32>, LoH: f32) -> vec3<f32> {
    // f_90 suitable for ambient occlusion
    // see https://google.github.io/filament/Filament.html#lighting/occlusion
    let f90 = saturate(dot(f0, vec3<f32>(50.0 * 0.33)));
    return F_Schlick_vec(f0, f90, LoH);
}

// Specular BRDF
// https://google.github.io/filament/Filament.html#materialsystem/specularbrdf

// Cook-Torrance approximation of the microfacet model integration using Fresnel law F to model f_m
// f_r(v,l) = { D(h,α) G(v,l,α) F(v,h,f0) } / { 4 (n⋅v) (n⋅l) }
fn specular(f0: vec3<f32>, roughness: f32, h: vec3<f32>, NoV: f32, NoL: f32, NoH: f32, LoH: f32, specularIntensity: f32) -> vec3<f32> {
    let D = D_GGX(roughness, NoH, h);
    let V = V_SmithGGXCorrelated(roughness, NoV, NoL);
    let F = fresnel(f0, LoH);

    return (specularIntensity * D * V) * F;
}

// Diffuse BRDF
// https://google.github.io/filament/Filament.html#materialsystem/diffusebrdf
// fd(v,l) = σ/π * 1 / { |n⋅v||n⋅l| } ∫Ω D(m,α) G(v,l,m) (v⋅m) (l⋅m) dm
//
// simplest approximation
// float Fd_Lambert() {
//     return 1.0 / PI;
// }
//
// vec3 Fd = diffuseColor * Fd_Lambert();
//
// Disney approximation
// See https://google.github.io/filament/Filament.html#citation-burley12
// minimal quality difference
fn Fd_Burley(roughness: f32, NoV: f32, NoL: f32, LoH: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * LoH * LoH;
    let lightScatter = F_Schlick(1.0, f90, NoL);
    let viewScatter = F_Schlick(1.0, f90, NoV);
    return lightScatter * viewScatter * (1.0 / PI);
}


fn point_light(
    world_position: vec3<f32>,
    light: LightData,
    roughness: f32,
    NdotV: f32,
    N: vec3<f32>,
    V: vec3<f32>,
    R: vec3<f32>,
    F0: vec3<f32>,
    diffuseColor: vec3<f32>
) -> vec3<f32> {
    var intensity = max(1000., light.intensity);
    intensity = intensity / (4.0 * PI);
    let range = max(2., light.range);
    let light_to_frag = light.position.xyz - world_position.xyz;
    let ligth_color_inverse_square_range = vec4<f32>(light.color.rgb * intensity, 1.0 / (range * range)) ;
    let distance_square = dot(light_to_frag, light_to_frag);
    let rangeAttenuation = getDistanceAttenuation(distance_square, ligth_color_inverse_square_range.w);

    // Specular.
    // Representative Point Area Lights.
    // see http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf p14-16
    let a = roughness;
    let centerToRay = dot(light_to_frag, R) * R - light_to_frag;
    let closestPoint = light_to_frag + centerToRay * saturate(range * inverseSqrt(dot(centerToRay, centerToRay)));
    let LspecLengthInverse = inverseSqrt(dot(closestPoint, closestPoint));
    let normalizationFactor = a / saturate(a + (range * 0.5 * LspecLengthInverse));
    let specularIntensity = normalizationFactor * normalizationFactor;

    var L: vec3<f32> = closestPoint * LspecLengthInverse; // normalize() equivalent?
    var H: vec3<f32> = normalize(L + V);
    var NoL: f32 = saturate(dot(N, L));
    var NoH: f32 = saturate(dot(N, H));
    var LoH: f32 = saturate(dot(L, H));

    let specular_light = specular(F0, roughness, H, NdotV, NoL, NoH, LoH, specularIntensity);

    // Diffuse.
    // Comes after specular since its NoL is used in the lighting equation.
    L = normalize(light_to_frag);
    H = normalize(L + V);
    NoL = saturate(dot(N, L));
    NoH = saturate(dot(N, H));
    LoH = saturate(dot(L, H));

    let diffuse = diffuseColor * Fd_Burley(roughness, NdotV, NoL, LoH);

    // See https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminanceEquation
    // Lout = f(v,l) Φ / { 4 π d^2 }⟨n⋅l⟩
    // where
    // f(v,l) = (f_d(v,l) + f_r(v,l)) * light_color
    // Φ is luminous power in lumens
    // our rangeAttentuation = 1 / d^2 multiplied with an attenuation factor for smoothing at the edge of the non-physical maximum light radius

    // For a point light, luminous intensity, I, in lumens per steradian is given by:
    // I = Φ / 4 π
    // The derivation of this can be seen here: https://google.github.io/filament/Filament.html#mjx-eqn-pointLightLuminousPower

    // NOTE: light.color.rgb is premultiplied with light.intensity / 4 π (which would be the luminous intensity) on the CPU

    // TODO compensate for energy loss https://google.github.io/filament/Filament.html#materialsystem/improvingthebrdfs/energylossinspecularreflectance

    return ((diffuse + specular_light) * ligth_color_inverse_square_range.rgb) * (rangeAttenuation * NoL);
}


@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    if (v_in.material_index < 0) {
        discard;
    }
    let material = dynamic_data.materials_data[v_in.material_index];

    var output_color = material.base_color;

    if (has_texture(v_in.material_index, TEXTURE_TYPE_BASE_COLOR)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_BASE_COLOR, v_in.tex_coords_base_color);
        output_color = output_color * t;
    }
    // TODO use .a for exposure compensation in HDR
    var emissive = vec4<f32>(material.emissive_color.rgb, 1.);
    if (has_texture(v_in.material_index, TEXTURE_TYPE_EMISSIVE)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_EMISSIVE, v_in.tex_coords_emissive);
        emissive = vec4<f32>(emissive.rgb * t.rgb, 1.) ;
    }
    // calculate non-linear roughness from linear perceptualRoughness
    var metallic = material.metallic_factor;
    var perceptual_roughness = material.roughness_factor;
    if (has_texture(v_in.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS, v_in.tex_coords_metallic_roughness);
        // Sampling from GLTF standard channels for now
        metallic = metallic * t.b;
        perceptual_roughness = perceptual_roughness * t.g;
    }
    let roughness = perceptualRoughnessToRoughness(perceptual_roughness);

    var occlusion = 1.0;
    if (has_texture(v_in.material_index, TEXTURE_TYPE_OCCLUSION)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_OCCLUSION, v_in.tex_coords_occlusion);
        occlusion = t.r;
    }

    var N = normalize(v_in.world_normal);
    var T = normalize(v_in.world_tangent.xyz - N * dot(v_in.world_tangent.xyz, N));
    var B = cross(N, T) * v_in.world_tangent.w;
    //if (!v_in.is_front_facing) {
    //            N = -N;
    //            T = -T;
    //            B = -B;
    //}
    let TBN = mat3x3<f32>(T, B, N);
    // Nt is the tangent-space normal.
    if (has_texture(v_in.material_index, TEXTURE_TYPE_NORMAL)) {
        var Nt = v_in.world_normal;
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_NORMAL, v_in.tex_coords_normal);
        Nt = t.rgb * 2.0 - 1.0;
        N = normalize(TBN * Nt);
    }

    if (material.alpha_mode == MATERIAL_ALPHA_BLEND_OPAQUE) {
        output_color.a = 1.0;
    } else if (material.alpha_mode == MATERIAL_ALPHA_BLEND_MASK) {
        if (output_color.a >= material.alpha_cutoff) {
            // NOTE: If rendering as masked alpha and >= the cutoff, render as fully opaque
            output_color.a = 1.0;
        } else {
            // NOTE: output_color.a < material.alpha_cutoff should not is not rendered
            // NOTE: This and any other discards mean that early-z testing cannot be done!
            discard;
        }
    } else if (material.alpha_mode == MATERIAL_ALPHA_BLEND_BLEND) {
        output_color.a = min(material.base_color.a, output_color.a);
        return output_color;
    }

    // Only valid for a perpective projection
    let view_pos = constant_data.view[3].xyz;
    let V = normalize(view_pos - v_in.world_position.xyz);

    // Neubelt and Pettineo 2013, "Crafting a Next-gen Material Pipeline for The Order: 1886"
    let NdotV = max(dot(N, V), 0.0001);

    // Remapping [0,1] reflectance to F0
    // See https://google.github.io/filament/Filament.html#materialsystem/parameterization/remapping
    let f0 = vec3<f32>(0.04, 0.04, 0.04);
    let specular_color = mix(f0, output_color.rgb, metallic);
    let reflectance = max(max(specular_color.r, specular_color.g), specular_color.b);
    let F0 = 0.16 * reflectance * reflectance * (1.0 - metallic) + output_color.rgb * metallic;

    // Diffuse strength inversely related to metallicity
    let diffuse_color = output_color.rgb * (1.0 - metallic);

    let R = reflect(-V, N);

    // accumulate color
    var light_accum: vec3<f32> = vec3<f32>(0.0);

    var i = 0u;
    loop {
        let light = dynamic_data.lights_data[i];
        if (dynamic_data.lights_data[i].light_type == 0u) {
            break;
        }
        let light_contrib = point_light(v_in.world_position.xyz, light, roughness, NdotV, N, V, R, F0, diffuse_color);
        light_accum = light_accum + light_contrib;

        i = i + 1u;
    }

    //TODO: Directional lights


    let diffuse_ambient = EnvBRDFApprox(diffuse_color, 1.0, NdotV);
    let specular_ambient = EnvBRDFApprox(F0, perceptual_roughness, NdotV);

    output_color = vec4<f32>(
        light_accum + (diffuse_ambient + specular_ambient) * AMBIENT_COLOR.rgb * occlusion + emissive.rgb * output_color.a,
        output_color.a
    );

    // tone_mapping
    output_color = vec4<f32>(reinhard_luminance(output_color.rgb), output_color.a);
    // Gamma correction.
    // Not needed with sRGB buffer
    //output_color = vec4<f32>(pow(output_color.rgb, vec3<f32>(1.0 / 2.2)), output_color.a);

    return output_color;
}