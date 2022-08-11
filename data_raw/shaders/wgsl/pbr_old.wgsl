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
let CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS: u32 = 2u;

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
    @builtin(vertex_index) index: u32,
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
    @builtin(instance_index) index: u32,
    @location(8) draw_area: vec4<f32>,
    @location(9) model_matrix_0: vec4<f32>,
    @location(10) model_matrix_1: vec4<f32>,
    @location(11) model_matrix_2: vec4<f32>,
    @location(12) model_matrix_3: vec4<f32>,
    @location(13) material_index: i32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) world_tangent: vec4<f32>,
    @location(4) view: vec3<f32>,
    @location(5) @interpolate(flat) material_index: i32,
    @location(6) tex_coords_base_color: vec2<f32>,
    @location(7) tex_coords_metallic_roughness: vec2<f32>,
    @location(8) tex_coords_normal: vec2<f32>,
    @location(9) tex_coords_emissive: vec2<f32>,
    @location(10) tex_coords_occlusion: vec2<f32>,
    @location(11) tex_coords_specular_glossiness: vec2<f32>,
    @location(12) tex_coords_diffuse: vec2<f32>,
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
var texture_array: binding_array<texture_2d_array<f32>, 8>; //MAX_TEXTURE_ATLAS_COUNT
#else
@group(1) @binding(2)
var texture_1: texture_2d_array<f32>;
@group(1) @binding(3)
var texture_2: texture_2d_array<f32>;
@group(1) @binding(4)
var texture_3: texture_2d_array<f32>;
@group(1) @binding(5)
var texture_4: texture_2d_array<f32>;
@group(1) @binding(6)
var texture_5: texture_2d_array<f32>;
@group(1) @binding(7)
var texture_6: texture_2d_array<f32>;
@group(1) @binding(8)
var texture_7: texture_2d_array<f32>;
@group(1) @binding(9)
var texture_8: texture_2d_array<f32>;
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

fn rand(n: u32) -> f32 {    
    // integer hash copied from Hugo Elias
    let n = (n << 13u) ^ n;
    let n = n * (n * n * 15731u + 789221u) + 1376312589u;
    return f32(n & u32(0x7fffffff)) / f32(0x7fffffff);
}

fn random_color(v: u32) -> vec3<f32> {
    let v1 = rand(v * 100u);
    let v2 = rand(v);
    let v3 = rand(u32(v1 - v2));
    return vec3(v1, v2, v3);
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
    let view_pos = constant_data.view[3].xyz;
    vertex_out.view = view_pos - vertex_out.world_position.xyz;
    vertex_out.color = v.color;
    vertex_out.material_index = instance.material_index;

    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let c = random_color(instance.index + v.index / 64u);
        vertex_out.color = vec4<f32>(c, 1.0);
    } else if (instance.material_index >= 0) {
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
    let layer_index = i32(dynamic_data.textures_data[texture_data_index].layer_index);

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
    return textureSampleLevel(texture_array[atlas_index], default_sampler, t.xy, layer_index, t.z);
#else
    if (atlas_index == 1u) {
        return textureSampleLevel(texture_2, default_sampler, t.xy, layer_index, t.z);
    } else if (atlas_index == 2u) {
        return textureSampleLevel(texture_3, default_sampler, t.xy, layer_index, t.z);
    } else if (atlas_index == 3u) {
        return textureSampleLevel(texture_4, default_sampler, t.xy, layer_index, t.z);
    } else if (atlas_index == 4u) {
        return textureSampleLevel(texture_5, default_sampler, t.xy, layer_index, t.z);
    } else if (atlas_index == 5u) {
        return textureSampleLevel(texture_6, default_sampler, t.xy, layer_index, t.z);
    } else if (atlas_index == 6u) {
        return textureSampleLevel(texture_7, default_sampler, t.xy, layer_index, t.z);
    } else if (atlas_index == 7u) {
        return textureSampleLevel(texture_8, default_sampler, t.xy, layer_index, t.z);
    }
    return textureSampleLevel(texture_1, default_sampler, t.xy, layer_index, t.z);
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


let PI: f32 = 3.141592653589793;
let AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.75, 0.75, 0.75);
let AMBIENT_INTENSITY = 0.45;
let NULL_VEC4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
let MinRoughness = 0.04;

// Find the normal for this fragment, pulling either from a predefined normal map
// or from the interpolated mesh normal and tangent attributes.
fn normal(v: VertexOutput) -> vec3<f32> {
    // Retrieve the tangent space matrix
    var n = normalize(v.world_normal);
    if (has_texture(v.material_index, TEXTURE_TYPE_NORMAL)) {
        var t = normalize(v.world_tangent.xyz - n * dot(v.world_tangent.xyz, n));
        var b = cross(n, t) * v.world_tangent.w;
        let tbn = mat3x3<f32>(t, b, n);
        let normal = get_texture_color(v.material_index, TEXTURE_TYPE_NORMAL, v.tex_coords_normal);
        n = tbn * (2.0 * normal.rgb - vec3<f32>(1.0));
        n = normalize(n);
    }
    
    //being front-facing culling we've to revert
    //n = -n;

    return n;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    if (v_in.material_index < 0) {
        discard;
    }

    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        return v_in.color;
    }

    let material = dynamic_data.materials_data[v_in.material_index];

    var base_color = min(v_in.color, material.base_color);
    if (has_texture(v_in.material_index, TEXTURE_TYPE_BASE_COLOR)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_BASE_COLOR, v_in.tex_coords_base_color);
        base_color = base_color * t;
    }
    
    // NOTE: the spec mandates to ignore any alpha value in 'OPAQUE' mode
    var alpha = 1.;
    if (material.alpha_mode == MATERIAL_ALPHA_BLEND_OPAQUE) {
        alpha = 1.0;
    } else if (material.alpha_mode == MATERIAL_ALPHA_BLEND_MASK) {
        if (alpha >= material.alpha_cutoff) {
            // NOTE: If rendering as masked alpha and >= the cutoff, render as fully opaque
            alpha = 1.0;
        } else {
            // NOTE: output_color.a < material.alpha_cutoff should not is not rendered
            // NOTE: This and any other discards mean that early-z testing cannot be done!
            discard;
        }
    } else if (material.alpha_mode == MATERIAL_ALPHA_BLEND_BLEND) {
        alpha = min(material.base_color.a, base_color.a);
    }
    
    // Metallic and Roughness material properties are packed together
    // In glTF, these factors can be specified by fixed scalar values
    // or from a metallic-roughness map
    var perceptual_roughness = material.roughness_factor;
    var metallic = material.metallic_factor;
    if (has_texture(v_in.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {
        let t = get_texture_color(v_in.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS, v_in.tex_coords_metallic_roughness);
        metallic = metallic * t.b;
        perceptual_roughness = perceptual_roughness * t.g;
    }
    perceptual_roughness = clamp(perceptual_roughness, MinRoughness, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);
    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness [2].
    let alpha_roughness = perceptual_roughness * perceptual_roughness;

    var ao = 1.0;
    var occlusion_strength = 1.;
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
    let NdotV = clamp(abs(dot(n, v)), 0.0001, 1.0);
    let reflection = reflect(-v, n);

    var color = base_color.rgb * AMBIENT_COLOR * AMBIENT_INTENSITY;
    color = mix(color, color * ao, occlusion_strength);
    color = color + emissive_color;

    var i = 0u;
    loop {
        let light = dynamic_data.lights_data[i];
        if (dynamic_data.lights_data[i].light_type == 0u) {
            break;
        }
        let l = normalize(light.position - v_in.world_position.xyz);             // Vector from surface point to light
        let h = normalize(l + v);                          // Half vector between both l and v
        let dist = length(light.position - v_in.world_position.xyz);                                // Distance from surface point to light

        let NdotL = clamp(dot(n, l), 0.0001, 1.0);
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
        var intensity = max(200., light.intensity);
        intensity = intensity / (4.0 * PI);
        let range = max(5., light.range);
        let light_contrib = (1. - min(dist / range, 1.)) * intensity;
        let diffuse_contrib = (1.0 - F) * diffuse(info);
        let spec_contrib = F * G * D / (4.0 * NdotL * NdotV);
        var light_color = NdotL * light.color.rgb * (diffuse_contrib + spec_contrib);
        light_color = light_color * light_contrib;

        color = color + light_color;

        i = i + 1u;
    }
    // TODO!: apply fix from reference shader:
    // https://github.com/KhronosGroup/glTF-WebGL-PBR/pull/55/files#diff-f7232333b020880432a925d5a59e075d
    let frag_color = vec4<f32>(color.rgb, alpha);
    return frag_color;
}