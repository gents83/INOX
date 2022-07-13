#import "utils.wgsl"
#import "common.wgsl"


struct PbrData {
    width: u32,
    height: u32,
    gbuffer_1: u32,
    gbuffer_2: u32,
    gbuffer_3: u32,
    depth: u32,
    _padding_2: u32,
    _padding_3: u32,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> pbr_data: PbrData;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> materials: Materials;
@group(0) @binding(4)
var<storage, read> textures: Textures;
@group(0) @binding(5)
var<storage, read> lights: Lights;

@group(1) @binding(0)
var render_target: texture_storage_2d_array<rgba8unorm, read_write>;



#import "texture_utils.wgsl"
#import "material_utils.wgsl"

fn get_uv(uvs: vec4<f32>, texture_index: u32, coords_set: u32) -> vec3<f32> {
    //var uv = unpack2x16float(u32(uvs.x));
    //if (coords_set == 1u) {
    //    uv = unpack2x16float(u32(uvs.y));
    //} else if (coords_set == 2u) {
    //    uv = unpack2x16float(u32(uvs.z));
    //} else if (coords_set == 3u) {
    //    uv = unpack2x16float(u32(uvs.w));
    //}
    var uv = uvs.xy;
    if (coords_set == 1u) {
        uv = uvs.zw;
    }
    return vec3<f32>(uv, f32(texture_index));
}

fn load(texture_index: u32, v: vec2<i32>) -> vec4<f32> {  
    return load_texture(vec3<i32>(v.xy, i32(texture_index)));
}


fn sample_material_texture(gbuffer_uvs: vec4<f32>, material_index: u32, texture_tyoe: u32) -> vec4<f32> {
    let material = &materials.data[material_index];
    let texture_id = u32((*material).textures_indices[texture_tyoe]);
    let coords_set = (*material).textures_coord_set[texture_tyoe];
    let uv = get_uv(gbuffer_uvs, texture_id, coords_set);
    return sample_texture(uv);
}

let PI: f32 = 3.141592653589793;
let AMBIENT_COLOR: vec3<f32> = vec3<f32>(0.75, 0.75, 0.75);
let AMBIENT_INTENSITY = 0.45;
let NULL_VEC4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
let MIN_ROUGHNESS = 0.04;

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



@compute
@workgroup_size(64, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let pixel = vec2<i32>(i32(global_invocation_id.x), i32(global_invocation_id.y));
    if (pixel.x >= i32(pbr_data.width) || pixel.y >= i32(pbr_data.height))
    {
        return;
    }
    
    let gbuffer_1 = load(pbr_data.gbuffer_1, pixel);
    let gbuffer_2 = load(pbr_data.gbuffer_2, pixel);

    var color = vec4<f32>(0., 0., 0., 0.);
    
    let mesh_id = u32(gbuffer_2.z);
    let vertex_color = u32(gbuffer_1.w);
    if mesh_id == 0u && vertex_color == 1u {
        textureStore(render_target, pixel.xy, 0, color);
        return;
    }

    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let meshlet_id = hash(u32(gbuffer_2.w));
        color = vec4<f32>(vec3<f32>(
            f32(meshlet_id & 255u), 
            f32((meshlet_id >> 8u) & 255u), 
            f32((meshlet_id >> 16u) & 255u)) / 255., 
            1.
        );
    } else {
        let gbuffer_3 = load(pbr_data.gbuffer_3, pixel);

        let material_id = u32(meshes.data[mesh_id].material_index);
        let material = &materials.data[material_id];
        let texture_color = sample_material_texture(gbuffer_3, material_id, TEXTURE_TYPE_BASE_COLOR);
        let vertex_color = unpack_unorm_to_4_f32(vertex_color);
        color = vec4<f32>(vertex_color.rgb * texture_color.rgb, vertex_color.a);

        let alpha = compute_alpha(material_id, vertex_color.a);
        if alpha < 0. {
            return;
        }
        
        var perceptual_roughness = (*material).roughness_factor;
        var metallic = (*material).metallic_factor;
        if (has_texture(material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {
            let t = sample_material_texture(gbuffer_3, material_id, TEXTURE_TYPE_METALLIC_ROUGHNESS);
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
            let t = sample_material_texture(gbuffer_3, material_id, TEXTURE_TYPE_OCCLUSION);
            ao = ao * t.r;
            occlusion_strength = (*material).occlusion_strength;
        }
        var emissive_color = vec3<f32>(0., 0., 0.);
        if (has_texture(material_id, TEXTURE_TYPE_EMISSIVE)) {
            let t = sample_material_texture(gbuffer_3, material_id, TEXTURE_TYPE_EMISSIVE);
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

        let n = unpack_normal(gbuffer_2.xy);
        let world_pos = gbuffer_1.xyz;
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
            var intensity = max(100., (*light).intensity);
            intensity = intensity / (4.0 * PI);
            let range = max(100., (*light).range);
            let light_contrib = (1. - min(dist / range, 1.)) * intensity;
            let diffuse_contrib = (1.0 - F) * diffuse_color / PI;
            let spec_contrib = F * G * D / (4.0 * NdotL * NdotV);
            var light_color = NdotL * (*light).color.rgb * (diffuse_contrib + spec_contrib);
            light_color = light_color * light_contrib;

            final_color = final_color + light_color;
        }
        
        color = vec4<f32>(final_color, color.a);
    }

    textureStore(render_target, pixel.xy, 0, color);
}