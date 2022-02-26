vec4 getTextureColor(uint texture_type) 
{
    uint texture_index = getAtlasIndex(in_material_index, texture_type);     
    if (texture_index == 0) {
        return texture(sampler2D(texture_1, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 1) {
        return texture(sampler2D(texture_2, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 2) {
        return texture(sampler2D(texture_3, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 3) {
        return texture(sampler2D(texture_4, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 4) {
        return texture(sampler2D(texture_5, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 5) {
        return texture(sampler2D(texture_6, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 6) {
        return texture(sampler2D(texture_7, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 7) {
        return texture(sampler2D(texture_8, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 8) {
        return texture(sampler2D(texture_9, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 9) {
        return texture(sampler2D(texture_10, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 10) {
        return texture(sampler2D(texture_11, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 11) {
        return texture(sampler2D(texture_12, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 12) {
        return texture(sampler2D(texture_13, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 13) {
        return texture(sampler2D(texture_14, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 14) {
        return texture(sampler2D(texture_15, default_sampler), in_tex_coord[texture_type].xy); 
    } else if (texture_index == 15) {
        return texture(sampler2D(texture_16, default_sampler), in_tex_coord[texture_type].xy); 
    } 
    return vec4(1.0, 1.0, 1.0, 1.0);
}
