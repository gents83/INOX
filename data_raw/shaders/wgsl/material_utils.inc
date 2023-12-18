fn has_texture(material: ptr<function, Material>, texture_type: u32) -> bool {
    if ((*material).textures_indices[texture_type] >= 0) {
        return true;
    }
    return false;
}

fn material_texture_index(material: ptr<function, Material>, texture_type: u32) -> u32 {
    let texture_index = (*material).textures_indices[texture_type];
    return u32(max(0, texture_index));
}

fn material_texture_coord_set(material: ptr<function, Material>, texture_type: u32) -> u32 {
    return (*material).textures_coord_set[texture_type];
}

fn material_texture_uv(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, texture_type: u32) -> vec3<f32> {
    let texture_coords_set = material_texture_coord_set(material, texture_type);  
    let texture_id = material_texture_index(material, texture_type);
    let uv = vec3<f32>((*pixel_data).uv_set[texture_coords_set], f32(texture_id));
    return uv;
} 

fn material_alpha(material_id: u32, vertex_color_alpha: f32) -> f32 {
    let material = &materials.data[material_id];
    
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

fn compute_tbn(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>) -> TBN {
    var n = normalize((*pixel_data).normal);
    var t = cross(n, vec3(1.,0.,1.));
    var b = normalize(cross(n,  t));
    t = normalize(cross(n, b));

    //if (has_texture(material, TEXTURE_TYPE_NORMAL)) {  
    //    let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_NORMAL);
    //    var uv_dx = dpdx(uv);
    //    var uv_dy = dpdy(uv);
    //    if (length(uv_dx) <= 0.01) {
    //        uv_dx = vec2<f32>(1., 0.);
    //    }
    //    if (length(uv_dy) <= 0.01) {
    //        uv_dy = vec2<f32>(0., 1.);
    //    }
    //    let t_temp = (uv_dy.x * dpdx((*pixel_data).world_pos) - uv_dx.x * dpdy((*pixel_data).world_pos)) /
    //                 (uv_dx.y * uv_dy.x - uv_dy.y * uv_dx.x);
    //
    //    t = normalize(t_temp - n * dot(n, t_temp));
    //    b = cross(n, t);
    //    
    //    var ntex = sample_texture(uv).rgb * 2. - vec3<f32>(1.);
    //    ntex = normalize(ntex);
    //    n = normalize(mat3x3<f32>(t, b, n) * ntex);
    //}
    return TBN(n, t, b);
}

fn compute_normal(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>) -> vec3<f32> {
    let tbn = compute_tbn(material, pixel_data);
    return tbn.normal;
}


struct MaterialInfo {
    base_color: vec4<f32>,
    ior: f32,
    f0: vec3<f32>,
    perceptual_roughness: f32,
    c_diff: vec3<f32>,
    metallic: f32,
    specular_weight: f32,
}


fn compute_base_color(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {
    (*info).base_color = (*material).diffuse_color;
    if (has_texture(material, TEXTURE_TYPE_DIFFUSE)) {  
        let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_DIFFUSE);
        let texture_color = sample_texture(uv);
        (*info).base_color *= texture_color;
    } else {
        (*info).base_color = (*material).base_color;
        if (has_texture(material, TEXTURE_TYPE_BASE_COLOR)) {  
            let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_BASE_COLOR);
            let texture_color = sample_texture(uv);
            (*info).base_color *= texture_color;
        }
    }
    (*info).base_color *= (*pixel_data).color;
}

fn compute_ior(material: ptr<function, Material>, info: ptr<function, MaterialInfo>) {
    (*info).ior = (*material).ior;
    (*info).f0 = vec3<f32>(pow(( (*info).ior - 1.) /  ((*info).ior + 1.), 2.));
}

fn compute_specular_glossiness(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {
    (*info).f0 = (*material).specular_color.xyz;
    (*info).perceptual_roughness = (*material).specular_color.w;
    if (has_texture(material, TEXTURE_TYPE_SPECULAR_GLOSSINESS)) {  
        let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_SPECULAR_GLOSSINESS);
        let texture_color = sample_texture(uv);
        (*info).f0 *= texture_color.rgb;
        (*info).perceptual_roughness *= texture_color.a;
    }
    (*info).perceptual_roughness = 1. - (*info).perceptual_roughness;
    (*info).c_diff = (*info).base_color.rgb * (1. - max(max((*info).f0.r, (*info).f0.g), (*info).f0.b));
}

fn compute_metallic_roughness(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {    
    (*info).perceptual_roughness = decode_snorm((u32((*material).roughness_metallic_factor) & 0xFFFF0000u) >> 16u, 16u);
    (*info).metallic = decode_snorm(u32((*material).roughness_metallic_factor) & 0x0000FFFFu, 16u);
    if (has_texture(material, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {        
        // Roughness is stored in the 'g' channel, metallic is stored in the 'b' channel.
        // This layout intentionally reserves the 'r' channel for (optional) occlusion map data
        let texture_color = material_texture_uv(material, pixel_data, TEXTURE_TYPE_METALLIC_ROUGHNESS);
        (*info).perceptual_roughness *= texture_color.g;
        (*info).metallic *= texture_color.b;
    }
    // Achromatic f0 based on IOR.
    (*info).c_diff = mix((*info).base_color.rgb,  vec3<f32>(0.), (*info).metallic);
    (*info).f0 = mix((*info).f0, (*info).base_color.rgb, (*info).metallic);
}

fn compute_sheen(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {   
    //NOT SUPPORTED TILL NOW - need gltf-rs support for KHR_materials_sheen
}

fn compute_clear_coat(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {   
    //NOT SUPPORTED TILL NOW - need gltf-rs support for KHR_materials_clearcoat
}

fn compute_specular(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {   
    var texture_color = vec4<f32>(1.0);
}