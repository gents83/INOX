fn has_texture(material_index: u32, texture_type: u32) -> bool {
    if (materials.data[material_index].textures_indices[texture_type] >= 0) {
        return true;
    }
    return false;
}

fn compute_uvs(material_index: u32, texture_type: u32, uv_0_1: vec4<f32>, uv_2_3: vec4<f32>) -> vec3<f32> {
   let material = &materials.data[material_index];    
    let texture_coords_set = (*material).textures_coord_set[texture_type];
    let texture_index = (*material).textures_indices[texture_type];
    var uv = uv_0_1.xy;
    if (texture_coords_set == 1u) {
        uv = uv_0_1.zw;
    } else if (texture_coords_set == 2u) {
        uv = uv_2_3.xy;
    } else if (texture_coords_set == 3u) {
        uv = uv_2_3.zw;
    } 
    return vec3<f32>(uv, f32(texture_index));
}

fn material_texture_index(material_index: u32, texture_type: u32) -> i32 {
    let material = &materials.data[material_index];
    return (*material).textures_indices[texture_type];
}

fn material_texture_coord_set(material_index: u32, texture_type: u32) -> u32 {
    let material = &materials.data[material_index];
    return (*material).textures_coord_set[texture_type];
}

fn sample_material_texture(uvs_0_1: vec4<f32>, material_index: u32, texture_type: u32) -> vec4<f32> {
    let texture_id = material_texture_index(material_index, texture_type);
    let coords_set = material_texture_coord_set(material_index, texture_type);
    let uv = get_uv(uvs_0_1, u32(texture_id), coords_set);
    return sample_texture(uv);
}