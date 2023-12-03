fn has_texture(material_id: u32, texture_type: u32) -> bool {
    let material = &materials.data[material_id];
    if ((*material).textures_indices[texture_type] >= 0) {
        return true;
    }
    return false;
}

fn material_texture_index(material_id: u32, texture_type: u32) -> u32 {
    let material = &materials.data[material_id];
    let texture_index = (*material).textures_indices[texture_type];
    if (texture_index < 0) {
        return 0u;
    }
    return u32(texture_index);
}

fn material_texture_coord_set(material_id: u32, texture_type: u32) -> u32 {
    let material = &materials.data[material_id];
    return (*material).textures_coord_set[texture_type];
}

fn material_texture_uv(pixel_data: ptr<function, PixelData>, texture_type: u32) -> vec3<f32> {
    let texture_coords_set = material_texture_coord_set((*pixel_data).material_id, texture_type);  
    let texture_id = material_texture_index((*pixel_data).material_id, texture_type);
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

fn compute_normal(pixel_data: ptr<function, PixelData>) -> vec3<f32> {
    var n = (*pixel_data).normal;
    if (has_texture((*pixel_data).material_id, TEXTURE_TYPE_NORMAL)) {    
        let uv = material_texture_uv(pixel_data, TEXTURE_TYPE_NORMAL);
        // get edge vectors of the pixel triangle 
        let dp1 = dpdx( (*pixel_data).world_pos ); 
        let dp2 = dpdy( (*pixel_data).world_pos ); 
        let duv1 = dpdx( uv.xy ); 
        let duv2 = dpdy( uv.xy );   
        // solve the linear system 
        let dp2perp = cross( dp2, n ); 
        let dp1perp = cross( n, dp1 ); 
        let tangent = dp2perp * duv1.x + dp1perp * duv2.x; 
        let bitangent = dp2perp * duv1.y + dp1perp * duv2.y;
        let t = normalize(tangent);
        let b = normalize(bitangent); 
        let tbn = mat3x3<f32>(t, b, n);
        let tn = sample_texture(uv);
        n = tbn * (2. * tn.rgb - vec3<f32>(1.));
        n = normalize(n);
    }
    return n;
}