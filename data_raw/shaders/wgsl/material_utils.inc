fn has_texture(material: ptr<function, Material>, texture_type: u32) -> bool {
    let i = texture_type & 3;  // Equivalent to index % 4 (faster than modulo on power of 2)
    let j = texture_type >> 2; // Equivalent to index / 4 (faster than division by power of 2)
    let texture_index = bitcast<u32>((*material).textures_index_and_coord_set[j][i]) & 0x0FFFFFFFu;
    if (texture_index > 0) {
        return true;
    }
    return false;
}

fn material_texture_index(material: ptr<function, Material>, texture_type: u32) -> u32 {
    let i = texture_type & 3;  // Equivalent to index % 4 (faster than modulo on power of 2)
    let j = texture_type >> 2; // Equivalent to index / 4 (faster than division by power of 2)
    let texture_index = bitcast<u32>((*material).textures_index_and_coord_set[j][i]) & 0x0FFFFFFFu;
    return max(0u, texture_index - 1u);
}

fn material_texture_coord_set(material: ptr<function, Material>, texture_type: u32) -> u32 {
    let i = texture_type & 3;  // Equivalent to index % 4 (faster than modulo on power of 2)
    let j = texture_type >> 2; // Equivalent to index / 4 (faster than division by power of 2)
    return (bitcast<u32>((*material).textures_index_and_coord_set[j][i]) & 0xF0000000u) >> 28;
}

fn material_texture_uv(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, texture_type: u32) -> vec3<f32> {
    let texture_coords_set = material_texture_coord_set(material, texture_type);  
    let texture_id = material_texture_index(material, texture_type);
    let uv = vec3<f32>((*pixel_data).uv_set[texture_coords_set].xy, f32(texture_id));
    return uv;
} 

fn compute_tbn(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>) -> TBN {
    var n = (*pixel_data).normal;
    var t = (*pixel_data).tangent.xyz;
    var b = cross(n,  t) * (*pixel_data).tangent.w;
    if (has_texture(material, TEXTURE_TYPE_NORMAL)) {  
        let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_NORMAL);
        let uv_set_index = material_texture_coord_set(material, TEXTURE_TYPE_NORMAL);
        var ntex = sample_texture(uv).rgb * 2. - vec3<f32>(1.);
        let normal_scale = unpack2x16float((*material).normal_scale_and_alpha_cutoff).x;
        ntex *= vec3<f32>(normal_scale, normal_scale, 1.);
        n = normalize(mat3x3<f32>(t, b, n) * normalize(ntex));
    }
    return TBN(n, t, b);
}

fn init_material_info_default(info: ptr<function, MaterialInfo>) {
    (*info).base_color = vec4<f32>(1.);
    (*info).f0 = vec3<f32>(0.04);
    (*info).ior = 1.5;
    (*info).c_diff = vec3<f32>(1.);
    (*info).perceptual_roughness = 1.;
    (*info).metallic = 1.;
    (*info).specular_weight_and_anisotropy_strength = pack2x16float(vec2<f32>(1., 0.));
    (*info).transmission_factor = 0.;
    (*info).thickness_factor = 0.;
    (*info).attenuation_color_and_distance = vec4<f32>(1.,1.,1.,0.);
    (*info).sheen_color_and_roughness_factor = vec4<f32>(1.,1.,1.,0.);
    (*info).clear_coat_factor = 0.;
    (*info).clear_coat_roughness_factor = 0.;
    (*info).iridescence_ior = 1.3;
    (*info).iridescence_thickness = 100.;
    (*info).alpha_roughness = 0.;
    (*info).f90 = vec3<f32>(1.);

    (*info).f_color = vec4<f32>(0.);
    (*info).f_emissive = vec3<f32>(0.);
    (*info).f_diffuse = vec3<f32>(0.);
    (*info).f_diffuse_ibl = vec3<f32>(0.);
    (*info).f_specular_ibl = vec3<f32>(0.);
    (*info).f_specular = vec3<f32>(0.);
    (*info).f_sheen = vec3<f32>(0.);
    (*info).f_sheen_ibl = vec3<f32>(0.);
    (*info).f_clearcoat = vec3<f32>(0.);
    (*info).f_clearcoat_ibl = vec3<f32>(0.);
    (*info).f_transmission = vec3<f32>(0.);
    (*info).shading_normal = vec3<f32>(0., 0., 1.);
}

fn compute_base_color(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {
    if(((*material).flags & MATERIAL_FLAGS_SPECULARGLOSSINESS) != 0u) {
        (*info).base_color = (*material).diffuse_color;
        if (has_texture(material, TEXTURE_TYPE_DIFFUSE)) {  
            let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_DIFFUSE);
            let texture_color = sample_texture(uv);
            (*info).base_color *= texture_color;
        }
    } else if(((*material).flags & MATERIAL_FLAGS_UNLIT) != 0u || ((*material).flags & MATERIAL_FLAGS_METALLICROUGHNESS) != 0u) {
        (*info).base_color = (*material).base_color;
        if (has_texture(material, TEXTURE_TYPE_BASE_COLOR)) {  
            let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_BASE_COLOR);
            let texture_color = sample_texture(uv);
            (*info).base_color *= texture_color;
        }
    }
    
    (*info).base_color *= (*pixel_data).color;
    if(((*material).flags & MATERIAL_FLAGS_ALPHAMODE_OPAQUE) != 0) {
        (*info).base_color.a = 1.;
    }
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
    (*info).perceptual_roughness = (*material).roughness_factor;
    (*info).metallic = (*material).metallic_factor;
    if (has_texture(material, TEXTURE_TYPE_METALLIC_ROUGHNESS)) {        
        // Roughness is stored in the 'g' channel, metallic is stored in the 'b' channel.
        // This layout intentionally reserves the 'r' channel for (optional) occlusion map data
        let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_METALLIC_ROUGHNESS);
        let texture_color = sample_texture(uv);
        (*info).perceptual_roughness *= texture_color.g;
        (*info).metallic *= texture_color.b;
    }
    // Achromatic f0 based on IOR.
    (*info).c_diff = mix((*info).base_color.rgb,  vec3<f32>(0.), (*info).metallic);
    (*info).f0 = mix((*info).f0, (*info).base_color.rgb, (*info).metallic);
}

fn compute_sheen(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {   
    (*info).sheen_color_and_roughness_factor = vec4<f32>(1.,1.,1.,0.);

    //NOT SUPPORTED TILL NOW - need gltf-rs support for KHR_materials_sheen
}

fn compute_clear_coat(material: ptr<function, Material>, normal: vec3<f32>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {   
    (*info).clear_coat_factor = 0.;
    (*info).clear_coat_roughness_factor = 0.;
    (*info).clear_coat_f0 = vec3<f32>(pow(((*info).ior - 1.0) / ((*info).ior + 1.0), 2.0));
    (*info).clear_coat_f90 = vec3<f32>(1.0);
    (*info).clear_coat_normal = normal;
    
    //NOT SUPPORTED TILL NOW - need gltf-rs support for KHR_materials_clearcoat
}

fn compute_specular(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {   
    var specular_texture = vec4<f32>(1.0);
    if (has_texture(material, TEXTURE_TYPE_SPECULAR)) {        
        let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_SPECULAR);
        let texture_color = sample_texture(uv);
        specular_texture.a = texture_color.a;
    }
    if (has_texture(material, TEXTURE_TYPE_SPECULAR_COLOR)) {        
        let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_SPECULAR_COLOR);
        let texture_color = sample_texture(uv);
        specular_texture = vec4<f32>(texture_color.rgb, specular_texture.a);
    }
    let dielectric_specular_f0 = min((*info).f0 * (*material).specular_factors.rgb * specular_texture.rgb, vec3<f32>(1.));
    let anisotropy_strength = unpack2x16float((*info).specular_weight_and_anisotropy_strength).y;
    (*info).specular_weight_and_anisotropy_strength = pack2x16float(vec2<f32>((*material).specular_factors.a * specular_texture.a, anisotropy_strength));
    (*info).f0 = mix(dielectric_specular_f0, (*info).base_color.rgb, (*info).metallic);
    (*info).c_diff = mix((*info).base_color.rgb, vec3<f32>(0.), (*info).metallic);
}

fn compute_transmission(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {   
    (*info).transmission_factor = (*material).transmission_factor;
    if (has_texture(material, TEXTURE_TYPE_TRANSMISSION)) {        
        let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_TRANSMISSION);
        let texture_color = sample_texture(uv);
        (*info).transmission_factor *= texture_color.r;
    }
}

fn compute_volume(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {   
    (*info).thickness_factor = (*material).thickness_factor;
    (*info).attenuation_color_and_distance = (*material).attenuation_color_and_distance;
    if (has_texture(material, TEXTURE_TYPE_THICKNESS)) {        
        let uv = material_texture_uv(material, pixel_data, TEXTURE_TYPE_THICKNESS);
        let texture_color = sample_texture(uv);
        (*info).thickness_factor *= texture_color.g;
    }
}

fn compute_iridescence(material: ptr<function, Material>, pixel_data: ptr<function, PixelData>, info: ptr<function, MaterialInfo>) {  
    (*info).iridescence_factor = 0.;
    (*info).iridescence_ior = (*info).ior;
    (*info).iridescence_thickness = 0.;

    //NOT SUPPORTED TILL NOW - need gltf-rs support for KHR_materials_iridescence 
}

fn compute_anisotropy(material: ptr<function, Material>, tbn: ptr<function, TBN>, info: ptr<function, MaterialInfo>) { 
    let specular_weight = unpack2x16float((*info).specular_weight_and_anisotropy_strength).x;
    (*info).anisotropicT = (*tbn).tangent;
    (*info).anisotropicB = (*tbn).binormal;
    (*info).specular_weight_and_anisotropy_strength = pack2x16float(vec2<f32>(specular_weight, 0.));

    //NOT SUPPORTED TILL NOW - need gltf-rs support for KHR_materials_anisotropy
}