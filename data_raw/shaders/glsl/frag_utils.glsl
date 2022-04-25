vec4 getTextureColor(uint texture_type) 
{
    uint texture_index = getAtlasIndex(in_material_index, texture_type);     
    return texture(sampler2D(textures[texture_index], default_sampler), in_tex_coord[texture_type].xy); 
}
