fn has_texture(material_index: u32, texture_type: u32) -> bool {
    if (materials.data[material_index].textures_indices[texture_type] >= 0) {
        return true;
    }
    return false;
}

fn material_texture_index(material_index: u32, texture_type: u32) -> i32 {
    let material = &materials.data[material_index];
    let texture_index = (*material).textures_indices[texture_type];
    if (texture_index < 0) {
        return 0;
    }
    return texture_index;
}

fn material_texture_coord_set(material_index: u32, texture_type: u32) -> u32 {
    let material = &materials.data[material_index];
    return (*material).textures_coord_set[texture_type];
}

fn get_uv(uv_set: vec4<u32>, texture_index: u32, coords_set: u32) -> vec3<f32> {
    var uv = vec2<f32>(0., 0.);
    switch (coords_set) {
        case 1u: { uv = unpack2x16float(uv_set.y); }
        case 2u: { uv = unpack2x16float(uv_set.z); }
        case 3u: { uv = unpack2x16float(uv_set.w); }
        default { uv = unpack2x16float(uv_set.x); }
    }
    return vec3<f32>(uv, f32(texture_index));
}

fn compute_uvs(material_index: u32, texture_type: u32, uv_set: vec4<u32>) -> vec3<f32> {
    let texture_id = material_texture_index(material_index, texture_type);
    let coords_set = material_texture_coord_set(material_index, texture_type);  
    let uv = get_uv(uv_set, u32(texture_id), coords_set);
    return uv;
}

fn sample_material_texture(material_index: u32, texture_type: u32, uv_set: vec4<u32>) -> vec4<f32> {
    let uv = compute_uvs(material_index, texture_type, uv_set);
    return sample_texture(uv);
}