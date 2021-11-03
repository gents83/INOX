uint getTextureIndex(int material_index, int texture_type) 
{
    int atlas_index = uniforms.material_data[material_index].textures_indices[texture_type];
    if (atlas_index >= 0 && atlas_index < uniforms.num_textures)
    {
        return uniforms.textures_data[atlas_index].texture_index;
    }
    return 0;
}

vec3 getTextureCoords(int material_index, int texture_type) 
{
    vec3 texcoords = vec3(0.);
    int atlas_index = uniforms.material_data[material_index].textures_indices[texture_type];
    if (atlas_index >= 0 && atlas_index < uniforms.num_textures)
    {
        uint layer_index = uniforms.textures_data[atlas_index].layer_index;
        vec4 area = uniforms.textures_data[atlas_index].area;
        vec2 texture_size = vec2(uniforms.textures_data[atlas_index].total_width, uniforms.textures_data[atlas_index].total_height);
        int textures_coord_set = uniforms.material_data[material_index].textures_coord_set[texture_type];
        if (textures_coord_set >= 0 && textures_coord_set < MAX_TEXTURE_COORDS_SETS)
        {
            texcoords.x = (area.x + 0.5 + vertex_tex_coord[textures_coord_set].x * area.z) / texture_size.x;
            texcoords.y = (area.x + 0.5 + vertex_tex_coord[textures_coord_set].y * area.w) / texture_size.y;
            texcoords.z = layer_index;
        }
    }
    return texcoords;
}
