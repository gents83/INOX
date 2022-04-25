
vec3 linearFromSrgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, cutoff);
}

vec4 rgbaFromInteger(uint color) {
    return vec4(
        float(color & 255), 
        float((color >> 8) & 255), 
        float((color >> 16) & 255), 
        float((color >> 24) & 255));
}

int getTextureDataIndex(int material_index, uint texture_type) 
{
    return dynamic_data.material_data[material_index].textures_indices[texture_type];
}

uint getAtlasIndex(int material_index, uint texture_type) 
{
    int index = getTextureDataIndex(material_index, texture_type);
    if (index < 0) {
        index = 0;
    } 
    uint atlas_index = dynamic_data.textures_data[index].texture_index;
    return atlas_index;
}

uint getTextureCoordsSet(int material_index, uint texture_type) 
{
    int textures_coord_set = dynamic_data.material_data[material_index].textures_coord_set[texture_type];
    if (textures_coord_set >= 0 && textures_coord_set < MAX_TEXTURE_COORDS_SETS)
    {
        return textures_coord_set;
    }
    return 0;
}

vec3 getTextureCoords(int material_index, uint texture_type, vec2 textures_coord_set) 
{
    vec3 texcoords = vec3(0.);
    int index = getTextureDataIndex(material_index, texture_type);
    if (index >= 0) 
    {
        vec4 area = dynamic_data.textures_data[index].area;
        
        texcoords.x = (area.x + 0.5 + textures_coord_set.x * area.z) / dynamic_data.textures_data[index].total_width;
        texcoords.y = (area.y + 0.5 + textures_coord_set.y * area.w) / dynamic_data.textures_data[index].total_height;
        texcoords.z = dynamic_data.textures_data[index].layer_index;
    }         
    return texcoords;
}
