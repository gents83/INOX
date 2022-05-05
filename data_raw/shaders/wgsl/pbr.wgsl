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

let CONSTANT_DATA_FLAGS_NONE: u32 = 0u;
let CONSTANT_DATA_FLAGS_SUPPORT_SRGB: u32 = 1u;

let PI = 3.14159265359;

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
    emissive_color: vec4<f32>,
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
    @location(0) world_pos: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) view: vec3<f32>,
    @location(6) @interpolate(flat) material_index: i32,
    @location(7) tex_coords_base_color: vec3<f32>,
    @location(8) tex_coords_metallic_roughness: vec3<f32>,
    @location(9) tex_coords_normal: vec3<f32>,
    @location(10) tex_coords_emissive: vec3<f32>,
    @location(11) tex_coords_occlusion: vec3<f32>,
    @location(12) tex_coords_specular_glossiness: vec3<f32>,
    @location(13) tex_coords_diffuse: vec3<f32>,
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
    let texture_data_index = dynamic_data.materials_data[material_index].textures_indices[texture_type];
    if (texture_data_index >= 0) {
        let textures_coord_set_index = dynamic_data.materials_data[material_index].textures_coord_set[texture_type];
        if (textures_coord_set_index == 1u) {
            return v.tex_coords_1;
        } else if (textures_coord_set_index == 2u) {
            return v.tex_coords_2;
        } else if (textures_coord_set_index == 3u) {
            return v.tex_coords_3;
        }
    }
    return v.tex_coords_0;
}


fn compute_textures_coord(v: VertexInput, material_index: i32, texture_type: u32) -> vec3<f32> {
    let tex_coords = get_textures_coord_set(v, material_index, texture_type);
    var t = vec3<f32>(0.0, 0.0, 0.0);
    let texture_data_index = dynamic_data.materials_data[material_index].textures_indices[texture_type];
    if (texture_data_index >= 0) {
        t.x = (dynamic_data.textures_data[texture_data_index].area.x + 0.5 + tex_coords.x * dynamic_data.textures_data[texture_data_index].area.z) / dynamic_data.textures_data[texture_data_index].total_width;
        t.y = (dynamic_data.textures_data[texture_data_index].area.y + 0.5 + tex_coords.y * dynamic_data.textures_data[texture_data_index].area.w) / dynamic_data.textures_data[texture_data_index].total_height;
        t.z = f32(dynamic_data.textures_data[texture_data_index].layer_index);
    }
    return t;
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
    vertex_out.world_pos = instance_matrix * vec4<f32>(v.position, 1.0);
    vertex_out.clip_position = constant_data.proj * constant_data.view * vertex_out.world_pos;
    vertex_out.normal = normalize((instance_matrix * vec4<f32>(v.normal, 0.0)).xyz);
    vertex_out.tangent = normalize((instance_matrix * vec4<f32>(v.tangent.xyz, 0.0)).xyz);
    vertex_out.bitangent = cross(vertex_out.normal, vertex_out.tangent) * v.tangent.w;
    let view_pos = vec3<f32>(constant_data.view[3][0], constant_data.view[3][1], constant_data.view[3][2]);
    vertex_out.view = view_pos - vertex_out.world_pos.xyz;
    vertex_out.color = v.color;
    vertex_out.material_index = instance.material_index;

    if (instance.material_index >= 0) {
        vertex_out.tex_coords_base_color = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_BASE_COLOR);
        vertex_out.tex_coords_metallic_roughness = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS);
        vertex_out.tex_coords_normal = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_NORMAL);
        vertex_out.tex_coords_emissive = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_EMISSIVE);
        vertex_out.tex_coords_occlusion = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_OCCLUSION);
        vertex_out.tex_coords_specular_glossiness = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_SPECULAR_GLOSSINESS);
        vertex_out.tex_coords_diffuse = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_DIFFUSE);
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

fn get_texture_color(material_index: i32, texture_type: u32, tex_coords: vec3<f32>) -> vec4<f32> {
    if (material_index < 0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let atlas_index = get_atlas_index(material_index, texture_type);   

#ifdef FEATURES_TEXTURE_BINDING_ARRAY
    return textureSampleLevel(texture_array[atlas_index], default_sampler, tex_coords.xy, tex_coords.z);
#else
    if (atlas_index == 1u) {
        return textureSampleLevel(texture_2, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 2u) {
        return textureSampleLevel(texture_3, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 3u) {
        return textureSampleLevel(texture_4, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 4u) {
        return textureSampleLevel(texture_5, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 5u) {
        return textureSampleLevel(texture_6, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 6u) {
        return textureSampleLevel(texture_7, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 7u) {
        return textureSampleLevel(texture_8, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 8u) {
        return textureSampleLevel(texture_9, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 9u) {
        return textureSampleLevel(texture_10, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 10u) {
        return textureSampleLevel(texture_11, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 11u) {
        return textureSampleLevel(texture_12, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 12u) {
        return textureSampleLevel(texture_13, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 13u) {
        return textureSampleLevel(texture_14, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 14u) {
        return textureSampleLevel(texture_15, default_sampler, tex_coords.xy, tex_coords.z);
    } else if (atlas_index == 15u) {
        return textureSampleLevel(texture_16, default_sampler, tex_coords.xy, tex_coords.z);
    }
    return textureSampleLevel(texture_1, default_sampler, tex_coords.xy, tex_coords.z);
#endif
}

struct SurfaceInfo {
    color: vec4<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    normal: vec3<f32>,
    f0: vec3<f32>,
    ao: f32,
    emissive: vec3<f32>,
    v: vec3<f32>
};

fn get_surface_info(v: VertexOutput) -> SurfaceInfo {
    var surface : SurfaceInfo;
    surface.v = normalize(v.view);
    surface.normal = normalize(v.normal);
    surface.ao = 1.0;
    surface.color = v.color;

    if (v.material_index < 0) {
        return surface;
    }
    let material = dynamic_data.materials_data[v.material_index];
    surface.color = surface.color * material.base_color;

    if (has_texture(v.material_index, TEXTURE_TYPE_BASE_COLOR)) {
        surface.color = surface.color * get_texture_color(v.material_index, TEXTURE_TYPE_BASE_COLOR, v.tex_coords_base_color);
        if (surface.color.a < 0.5) {
            discard;
        }
    }

    surface.albedo = surface.color.rgb;
    surface.emissive = material.emissive_color.rgb;
    surface.metallic = material.metallic_factor;
    surface.roughness = material.roughness_factor;

    if (has_texture(v.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {
        let metallic_roughness = get_texture_color(v.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS, v.tex_coords_metallic_roughness);
        surface.metallic = surface.metallic * metallic_roughness.b;
        surface.roughness = surface.roughness * metallic_roughness.g;
    }

    if (has_texture(v.material_index, TEXTURE_TYPE_NORMAL)) {
        let tbn = mat3x3<f32>(v.tangent, v.bitangent, v.normal);
        let normal = get_texture_color(v.material_index, TEXTURE_TYPE_NORMAL, v.tex_coords_normal);
        surface.normal = normalize(tbn * (2.0 * normal.xyz - vec3<f32>(1.0, 1.0, 1.0)));
    }

    let dielectric_specular = vec3<f32>(0.04, 0.04, 0.04);
    surface.f0 = mix(dielectric_specular, surface.albedo, vec3<f32>(surface.metallic, surface.metallic, surface.metallic));

    if (has_texture(v.material_index, TEXTURE_TYPE_OCCLUSION)) {
        let ao = get_texture_color(v.material_index, TEXTURE_TYPE_OCCLUSION, v.tex_coords_occlusion);
        surface.ao = ao.r * material.alpha_cutoff;
    }

    if (has_texture(v.material_index, TEXTURE_TYPE_EMISSIVE)) {
        let emissive = get_texture_color(v.material_index, TEXTURE_TYPE_EMISSIVE, v.tex_coords_emissive);
        surface.emissive = surface.emissive * emissive.rgb;
    }

    return surface;
}


fn compute_fresnel_schlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3<f32>(1.0, 1.0, 1.0) - F0) * pow(1.0 - cosTheta, 5.0);
}
fn compute_distribution_GGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let num = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return num / (PI * denom * denom);
}
fn compute_geometry_schlick_GGX(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;
    return num / denom;
}
fn compute_geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = compute_geometry_schlick_GGX(NdotV, roughness);
    let ggx1 = compute_geometry_schlick_GGX(NdotL, roughness);
    return ggx1 * ggx2;
}
fn compute_range_attenuation(range: f32, distance: f32) -> f32 {
    if (range <= 0.0) {
      // Negative range means no cutoff
        return 1.0 / pow(distance, 2.0);
    }
    return clamp(1.0 - pow(distance / range, 4.0), 0.0, 1.0) / pow(distance, 2.0);
}
fn compute_light_radiance(vertex_pos: vec3<f32>, light: LightData, surface: SurfaceInfo) -> vec3<f32> {
    let point_to_light = light.position - vertex_pos;
    let L = normalize(point_to_light);
    let H = normalize(surface.v + L);
    let distance = length(point_to_light);
  // cook-torrance brdf
    let NDF = compute_distribution_GGX(surface.normal, H, surface.roughness);
    let G = compute_geometry_smith(surface.normal, surface.v, L, surface.roughness);
    let F = compute_fresnel_schlick(max(dot(H, surface.v), 0.0), surface.f0);
    let kD = (vec3<f32>(1.0, 1.0, 1.0) - F) * (1.0 - surface.metallic);
    let NdotL = max(dot(surface.normal, L), 0.0);
    let numerator = NDF * G * F;
    let denominator = max(4.0 * max(dot(surface.normal, surface.v), 0.0) * NdotL, 0.001);
    let specular = numerator / vec3<f32>(denominator, denominator, denominator);
    let intensity = 1.;//light.intensity
    // add to outgoing radiance Lo
    let attenuation = compute_range_attenuation(light.range, distance);
    let radiance = light.color.rgb * intensity * attenuation;
    return (kD * surface.albedo / vec3<f32>(PI, PI, PI) + specular) * radiance * NdotL;
}

  // linear <-> sRGB conversions
fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    if (all(color <= vec3<f32>(0.0031308, 0.0031308, 0.0031308))) {
        return color * 12.92;
    }
    return (pow(abs(color), vec3<f32>(1.0 / 2.4, 1.0 / 2.4, 1.0 / 2.4)) * 1.055) - vec3<f32>(0.055, 0.055, 0.055);
}
fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    if (all(color <= vec3<f32>(0.04045, 0.04045, 0.04045))) {
        return color / vec3<f32>(12.92, 12.92, 12.92);
    }
    return pow((color + vec3<f32>(0.055, 0.055, 0.055)) / vec3<f32>(1.055, 1.055, 1.055), vec3<f32>(2.4, 2.4, 2.4));
}

@fragment
fn fs_main(v: VertexOutput) -> @location(0) vec4<f32> {
    let surface = get_surface_info(v);
    
    // reflectance equation
    var ambient_light = vec3<f32>(1.);
    var color_from_light = vec3<f32>(0.0, 0.0, 0.0);

    var i = 0u;
    loop {
        if (dynamic_data.lights_data[i].light_type == 0u) {
            break;
        }
        
        // calculate per-light radiance and add to outgoing radiance Lo
        color_from_light = color_from_light + compute_light_radiance(v.world_pos.xyz, dynamic_data.lights_data[i], surface);
        i = i + 1u;
    }

    let ambient = ambient_light * surface.albedo * surface.ao;
    let color = linear_to_srgb(color_from_light + ambient + surface.emissive);
    return vec4<f32>(color, surface.color.a);
}