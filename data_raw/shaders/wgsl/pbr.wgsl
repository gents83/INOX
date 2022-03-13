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

struct ConstantData {
    view: mat4x4<f32>;
    proj: mat4x4<f32>;
    screen_width: f32;
    screen_height: f32;
    flags: u32;
};

struct LightData {
    position: vec3<f32>;
    light_type: u32;
    color: vec4<f32>;
    intensity: f32;
    range: f32;
    inner_cone_angle: f32;
    outer_cone_angle: f32;
};

struct TextureData {
    texture_index: u32;
    layer_index: u32;
    total_width: f32;
    total_height: f32;
    area: vec4<f32>;
};

struct ShaderMaterialData {
    textures_indices: array<i32, 8>;//TEXTURE_TYPE_COUNT>;
    textures_coord_set: array<u32, 8>;//TEXTURE_TYPE_COUNT>;
    roughness_factor: f32;
    metallic_factor: f32;
    alpha_cutoff: f32;
    alpha_mode: u32;
    base_color: vec4<f32>;
    emissive_color: vec4<f32>;
    diffuse_color: vec4<f32>;
    specular_color: vec4<f32>;
};

struct DynamicData {
    lights_data: array<LightData, 64>;//MAX_NUM_LIGHTS>;
    textures_data: array<TextureData, 512>;//MAX_NUM_TEXTURES>;
    materials_data: array<ShaderMaterialData, 512>;//MAX_NUM_MATERIALS>;
    num_lights: u32;
};


struct VertexInput {
    //@builtin(vertex_index) index: u32;
    @location(0) position: vec3<f32>;
    @location(1) normal: vec3<f32>;
    @location(2) color: vec4<f32>;
    @location(3) tex_coords_0: vec2<f32>;
    @location(4) tex_coords_1: vec2<f32>;
    @location(5) tex_coords_2: vec2<f32>;
    @location(6) tex_coords_3: vec2<f32>;
};

struct InstanceInput {
    //@builtin(instance_index) index: u32;
    @location(7) draw_area: vec4<f32>;
    @location(8) model_matrix_0: vec4<f32>;
    @location(9) model_matrix_1: vec4<f32>;
    @location(10) model_matrix_2: vec4<f32>;
    @location(11) model_matrix_3: vec4<f32>;
    @location(12) normal_matrix_0: vec3<f32>;
    @location(13) normal_matrix_1: vec3<f32>;
    @location(14) normal_matrix_2: vec3<f32>;
    @location(15) material_index: i32;
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>;
    @location(0) color: vec4<f32>;
    @location(1) normal: vec3<f32>;
    @location(2) @interpolate(flat) material_index: i32;
    @location(3) tex_coords_base_color: vec3<f32>;
    @location(4) tex_coords_metallic_roughness: vec3<f32>;
    @location(5) tex_coords_normal: vec3<f32>;
    @location(6) tex_coords_emissive: vec3<f32>;
    @location(7) tex_coords_occlusion: vec3<f32>;
    @location(8) tex_coords_specular_glossiness: vec3<f32>;
    @location(9) tex_coords_diffuse: vec3<f32>;
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> dynamic_data: DynamicData;

@group(1) @binding(0)
var default_sampler: sampler;
@group(1) @binding(1)
var texture_1: texture_2d<f32>;
@group(1) @binding(2)
var texture_2: texture_2d<f32>;
@group(1) @binding(3)
var texture_3: texture_2d<f32>;
@group(1) @binding(4)
var texture_4: texture_2d<f32>;
@group(1) @binding(5)
var texture_5: texture_2d<f32>;
@group(1) @binding(6)
var texture_6: texture_2d<f32>;
@group(1) @binding(7)
var texture_7: texture_2d<f32>;
@group(1) @binding(8)
var texture_8: texture_2d<f32>;
@group(1) @binding(9)
var texture_9: texture_2d<f32>;
@group(1) @binding(10)
var texture_10: texture_2d<f32>;
@group(1) @binding(11)
var texture_11: texture_2d<f32>;
@group(1) @binding(12)
var texture_12: texture_2d<f32>;
@group(1) @binding(13)
var texture_13: texture_2d<f32>;
@group(1) @binding(14)
var texture_14: texture_2d<f32>;
@group(1) @binding(15)
var texture_15: texture_2d<f32>;
@group(1) @binding(16)
var texture_16: texture_2d<f32>;

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
    var output = vec3<f32>(0.0, 0.0, 0.0); 
    let texture_data_index = dynamic_data.materials_data[material_index].textures_indices[texture_type];
    if (texture_data_index >= 0) {
        output.x = (dynamic_data.textures_data[texture_data_index].area.x + 0.5 + tex_coords.x * dynamic_data.textures_data[texture_data_index].area.z) / dynamic_data.textures_data[texture_data_index].total_width;
        output.y = (dynamic_data.textures_data[texture_data_index].area.y + 0.5 + tex_coords.y * dynamic_data.textures_data[texture_data_index].area.w) / dynamic_data.textures_data[texture_data_index].total_height;
        output.z = f32(dynamic_data.textures_data[texture_data_index].layer_index);
    }
    return output;
}

@stage(vertex)
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
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    var out: VertexOutput;
    out.clip_position = constant_data.proj * constant_data.view * instance_matrix * vec4<f32>(v.position, 1.0);
    out.normal = normal_matrix * v.normal;
    out.color = v.color;
    out.material_index = instance.material_index;

    if (instance.material_index >= 0)
    {
        out.tex_coords_base_color = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_BASE_COLOR);
        out.tex_coords_metallic_roughness = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_METALLIC_ROUGHNESS);
        out.tex_coords_normal = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_NORMAL);
        out.tex_coords_emissive = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_EMISSIVE);
        out.tex_coords_occlusion = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_OCCLUSION);
        out.tex_coords_specular_glossiness = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_SPECULAR_GLOSSINESS);
        out.tex_coords_diffuse = compute_textures_coord(v, instance.material_index, TEXTURE_TYPE_DIFFUSE);
    }

    return out;
}

fn get_atlas_index(material_index: u32, texture_type: u32) -> u32 {
    let texture_data_index = dynamic_data.materials_data[material_index].textures_indices[texture_type];
    if (texture_data_index < 0) {
        return 0u;
    }
    return dynamic_data.textures_data[texture_data_index].texture_index;
}

fn get_texture_color(material_index: u32, texture_type: u32, tex_coords: vec3<f32>) -> vec4<f32> {
    let atlas_index = get_atlas_index(material_index, texture_type);
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
}


@stage(fragment)
fn fs_main(v: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = v.color;
    if (v.material_index >= 0)
    {
        color = color * get_texture_color(u32(v.material_index), TEXTURE_TYPE_BASE_COLOR, v.tex_coords_base_color);  
    }

    var color_from_light = color.rgb;
    var i = 0u;
	loop {
        if (i >= dynamic_data.num_lights) {
            break;
        }
        let ambient_strength = dynamic_data.lights_data[i].intensity / 10000.;
	    let ambient_color = dynamic_data.lights_data[i].color.rgb * ambient_strength;

	    let light_dir = normalize(dynamic_data.lights_data[i].position - v.clip_position.xyz);
	    
	    let diffuse_strength = max(dot(v.normal, light_dir), 0.0);
	    let diffuse_color = dynamic_data.lights_data[i].color.rgb * diffuse_strength;
	    let view_pos = vec3<f32>(constant_data.view[3][0], constant_data.view[3][1], constant_data.view[3][2]);
	    let view_dir = normalize(view_pos - v.clip_position.xyz);

	    //Blinn-Phong
	    let half_dir = normalize(view_dir + light_dir);
	    let specular_strength = pow(max(dot(v.normal, half_dir), 0.0), 32.);

	    let specular_color = specular_strength * dynamic_data.lights_data[i].color.rgb;

	    color_from_light = color_from_light * (ambient_color + diffuse_color + specular_color);
        i = i + 1u;
	}
    
    return vec4<f32>(color_from_light.rgb, color.a);
}