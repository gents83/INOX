let MAX_TEXTURE_ATLAS_COUNT: u32 = 16u;
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
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) view: vec3<f32>,
    @location(6) @interpolate(flat) material_index: i32,
    @location(7) tex_coords_base_color: vec2<f32>,
    @location(8) tex_coords_metallic_roughness: vec2<f32>,
    @location(9) tex_coords_normal: vec2<f32>,
    @location(10) tex_coords_emissive: vec2<f32>,
    @location(11) tex_coords_occlusion: vec2<f32>,
    @location(12) tex_coords_specular_glossiness: vec2<f32>,
    @location(13) tex_coords_diffuse: vec2<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> dynamic_data: DynamicData;

@group(1) @binding(0)
var default_sampler: sampler;
@group(1) @binding(1)
var depth_sampler: sampler;

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
@group(1) @binding(10)
var texture_9: texture_2d<f32>;
@group(1) @binding(11)
var texture_10: texture_2d<f32>;
@group(1) @binding(12)
var texture_11: texture_2d<f32>;
@group(1) @binding(13)
var texture_12: texture_2d<f32>;
@group(1) @binding(14)
var texture_13: texture_2d<f32>;
@group(1) @binding(15)
var texture_14: texture_2d<f32>;
@group(1) @binding(16)
var texture_15: texture_2d<f32>;
@group(1) @binding(17)
var texture_16: texture_2d<f32>;
#endif

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

    var vertex_out: VertexOutput;
    vertex_out.position = instance_matrix * vec4<f32>(v.position, 1.0);
    vertex_out.clip_position = constant_data.proj * constant_data.view * vertex_out.position;
    vertex_out.normal = normalize((instance_matrix * vec4<f32>(v.normal, 0.0)).xyz);
    vertex_out.tangent = normalize((instance_matrix * vec4<f32>(v.tangent.xyz, 0.0)).xyz);
    vertex_out.bitangent = cross(vertex_out.normal, vertex_out.tangent) * v.tangent.w;
    let view_pos = vec3<f32>(constant_data.view[3][0], constant_data.view[3][1], constant_data.view[3][2]);
    vertex_out.view = view_pos - vertex_out.position.xyz;
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

fn get_atlas_index(material_index: i32, texture_type: u32) -> u32 {
    if (material_index < 0) {
        return 0u;
    }
    let texture_data_index = dynamic_data.materials_data[material_index].textures_indices[texture_type];
    if (texture_data_index < 0) {
        return 0u;
    }
    return dynamic_data.textures_data[texture_data_index].texture_index;
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
    let atlas_index = dynamic_data.textures_data[texture_data_index].texture_index;
    var t = vec3<f32>(0.0, 0.0, 0.0);
    if (texture_data_index >= 0) {
        let area = dynamic_data.textures_data[texture_data_index].area;
        let image_width = dynamic_data.textures_data[texture_data_index].total_width;
        let image_height = dynamic_data.textures_data[texture_data_index].total_height;
        t.x = (area.x + 0.5 + area.z * fract(tex_coords.x)) / image_width;
        t.y = (area.y + 0.5 + area.w * fract(tex_coords.y)) / image_height;
        t.z = f32(dynamic_data.textures_data[texture_data_index].layer_index);
    }
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
    } else if (atlas_index == 9u) {
        return textureSampleLevel(texture_10, default_sampler, t.xy, t.z);
    } else if (atlas_index == 10u) {
        return textureSampleLevel(texture_11, default_sampler, t.xy, t.z);
    } else if (atlas_index == 11u) {
        return textureSampleLevel(texture_12, default_sampler, t.xy, t.z);
    } else if (atlas_index == 12u) {
        return textureSampleLevel(texture_13, default_sampler, t.xy, t.z);
    } else if (atlas_index == 13u) {
        return textureSampleLevel(texture_14, default_sampler, t.xy, t.z);
    } else if (atlas_index == 14u) {
        return textureSampleLevel(texture_15, default_sampler, t.xy, t.z);
    } else if (atlas_index == 15u) {
        return textureSampleLevel(texture_16, default_sampler, t.xy, t.z);
    }
    return textureSampleLevel(texture_1, default_sampler, t.xy, t.z);
#endif
}

// References:
// [1] Real Shading in Unreal Engine 4
//     http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
// [2] Physically Based Shading at Disney
//     http://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_notes_v3.pdf
// [3] README.md - Environment Maps
//     https://github.com/KhronosGroup/glTF-WebGL-PBR/#environment-maps
// [4] "An Inexpensive BRDF Model for Physically based Rendering" by Christophe Schlick
//     https://www.cs.virginia.edu/~jdl/bib/appearance/analytic%20models/schlick94b.pdf
struct PBRInfo {
    NdotL: f32,                  // cos angle between normal and light direction
    NdotV: f32,                  // cos angle between normal and view direction
    NdotH: f32,                  // cos angle between normal and half vector
    LdotH: f32,                  // cos angle between light direction and half vector
    VdotH: f32,                  // cos angle between view direction and half vector
    perceptual_roughness: f32,   // roughness value, as authored by the model creator (input to shader)
    metalness: f32,              // metallic value at the surface
    alpha_roughness: f32,        // roughness mapped to a more linear change in the roughness (proposed by [2])
    reflectance0: vec3<f32>,     // full reflectance color (normal incidence angle)
    reflectance90: vec3<f32>,    // reflectance color at grazing angle
    diffuse_color: vec3<f32>,    // color contribution from diffuse lighting
    specular_color: vec3<f32>,   // color contribution from specular lighting
};

let PI = 3.14159265359;
let MinRoughness = 0.04;
let AmbientLightColor = vec3<f32>(1., 1., 1.);
let AmbientLightIntensity = 0.2;

// Find the normal for this fragment, pulling either from a predefined normal map
// or from the interpolated mesh normal and tangent attributes.
fn normal(v: VertexOutput) -> vec3<f32> {
    // Retrieve the tangent space matrix
    let tbn = mat3x3<f32>(v.tangent, v.bitangent, v.normal);
    var n = v.normal;
    if (has_texture(v.material_index, TEXTURE_TYPE_NORMAL)) {
        let tbn = mat3x3<f32>(v.tangent, v.bitangent, v.normal);
        let normal = get_texture_color(v.material_index, TEXTURE_TYPE_NORMAL, v.tex_coords_normal);
        n = tbn * (2.0 * normal.xyz - vec3<f32>(1.0));
    }
    n = normalize(n);
    
    //being front-facing culling we've to revert
    n = -n;

    return n;
}
// Basic Lambertian diffuse
// Implementation from Lambert's Photometria https://archive.org/details/lambertsphotome00lambgoog
// See also [1], Equation 1
fn diffuse(info: PBRInfo) -> vec3<f32> {
    return info.diffuse_color / PI;
}
// The following equation models the Fresnel reflectance term of the spec equation (aka F())
// Implementation of fresnel from [4], Equation 15
fn specular_reflection(info: PBRInfo) -> vec3<f32> {
    return info.reflectance0 + (info.reflectance90 - info.reflectance0) * pow(clamp(1.0 - info.VdotH, 0.0, 1.0), 5.0);
}
// This calculates the specular geometric attenuation (aka G()),
// where rougher material will reflect less light back to the viewer.
// This implementation is based on [1] Equation 4, and we adopt their modifications to
// alphaRoughness as input as originally proposed in [2].
fn geometric_occlusion(info: PBRInfo) -> f32 {
    let r = info.alpha_roughness;

    let attenuationL = 2.0 * info.NdotL / (info.NdotL + sqrt(r * r + (1.0 - r * r) * (info.NdotL * info.NdotL)));
    let attenuationV = 2.0 * info.NdotV / (info.NdotV + sqrt(r * r + (1.0 - r * r) * (info.NdotV * info.NdotV)));
    return attenuationL * attenuationV;
}

// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
fn microfacet_distribution(info: PBRInfo) -> f32 {
    let roughnessSq = info.alpha_roughness * info.alpha_roughness;
    let f = (info.NdotH * roughnessSq - info.NdotH) * info.NdotH + 1.0;
    return roughnessSq / (PI * f * f);
}


@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    if (v_in.material_index < 0) {
        discard;
    }
    let material = dynamic_data.materials_data[v_in.material_index];
    
    // NOTE: the spec mandates to ignore any alpha value in 'OPAQUE' mode
    var alpha_blend = 0.;
    if (material.alpha_mode != MATERIAL_ALPHA_BLEND_OPAQUE) {
        alpha_blend = 1.;
    }
    var alpha = mix(1.0, material.base_color.a, alpha_blend);
    if (material.alpha_cutoff > 0.0 && material.alpha_mode == MATERIAL_ALPHA_BLEND_MASK) {
        alpha = step(material.alpha_cutoff, material.base_color.a);
    }
    if (alpha == 0.0) {
        discard;
    }

    // Metallic and Roughness material properties are packed together
    // In glTF, these factors can be specified by fixed scalar values
    // or from a metallic-roughness map
    var perceptual_roughness = material.roughness_factor;
    var metallic = material.metallic_factor;
    if (has_texture(v_in.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS, v_in.tex_coords_metallic_roughness);
        perceptual_roughness = perceptual_roughness * t.g;
        metallic = metallic * t.b;
    }
    perceptual_roughness = clamp(perceptual_roughness, MinRoughness, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);
    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness [2].
    let alpha_roughness = perceptual_roughness * perceptual_roughness;

    var base_color = v_in.color * material.base_color;
    if (has_texture(v_in.material_index, TEXTURE_TYPE_BASE_COLOR)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_BASE_COLOR, v_in.tex_coords_base_color);
        base_color = base_color * t;
    }
    var ao = 1.0;
    var occlusion_strength = 0.;
    if (has_texture(v_in.material_index, TEXTURE_TYPE_OCCLUSION)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_OCCLUSION, v_in.tex_coords_occlusion);
        ao = ao * t.r;
        occlusion_strength = material.occlusion_strength;
    }
    var emissive_color = vec3<f32>(0., 0., 0.);
    if (has_texture(v_in.material_index, TEXTURE_TYPE_EMISSIVE)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_EMISSIVE, v_in.tex_coords_emissive);
        emissive_color = t.rgb * material.emissive_color;
    }

    let f0 = vec3<f32>(0.04, 0.04, 0.04);
    var diffuse_color = base_color.rgb * (vec3<f32>(1., 1., 1.) - f0);
    diffuse_color = diffuse_color * (1.0 - metallic);
    let specular_color = mix(f0, base_color.rgb, metallic);

    // Compute reflectance.
    let reflectance = max(max(specular_color.r, specular_color.g), specular_color.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    let reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    let specular_environmentR0 = specular_color.rgb;
    let specular_environmentR90 = vec3<f32>(1., 1., 1.) * reflectance90;

    let n = normal(v_in);                             // normal at surface point
    let v = normalize(v_in.view);        // Vector from surface point to camera
    let NdotV = clamp(abs(dot(n, v)), 0.001, 1.0);
    let reflection = -normalize(reflect(v, n));

    var color = AmbientLightColor * AmbientLightIntensity * base_color.xyz;
    color = mix(color, color * ao, occlusion_strength);
    color = color + emissive_color;

    var i = 0u;
    loop {
        let light = dynamic_data.lights_data[i];
        if (dynamic_data.lights_data[i].light_type == 0u) {
            break;
        }
        let l = normalize(light.position - v_in.position.xyz);             // Vector from surface point to light
        let h = normalize(l + v);                          // Half vector between both l and v

        let NdotL = clamp(dot(n, l), 0.001, 1.0);
        let NdotH = clamp(dot(n, h), 0.0, 1.0);
        let LdotH = clamp(dot(l, h), 0.0, 1.0);
        let VdotH = clamp(dot(v, h), 0.0, 1.0);

        let info = PBRInfo(
            NdotL,
            NdotV,
            NdotH,
            LdotH,
            VdotH,
            perceptual_roughness,
            metallic,
            alpha_roughness,
            specular_environmentR0,
            specular_environmentR90,
            diffuse_color,
            specular_color
        );
        
        // Calculate the shading terms for the microfacet specular shading model
        let F = specular_reflection(info);
        let G = geometric_occlusion(info);
        let D = microfacet_distribution(info);

        // Calculation of analytical lighting contribution
        let diffuse_contrib = (1.0 - F) * diffuse(info);
        let spec_contrib = F * G * D / (4.0 * NdotL * NdotV);
        let light_color = NdotL * light.color.rgb * (diffuse_contrib + spec_contrib);

        color = color + light_color;

        i = i + 1u;
    }
    // TODO!: apply fix from reference shader:
    // https://github.com/KhronosGroup/glTF-WebGL-PBR/pull/55/files#diff-f7232333b020880432a925d5a59e075d
    let frag_color = vec4<f32>(color.rgb, alpha);
    return frag_color;
}