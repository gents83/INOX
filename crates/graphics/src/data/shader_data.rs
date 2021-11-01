use crate::{
    print_field_size, LightData, ShaderMaterialData, ShaderTextureData, MAX_NUM_LIGHTS,
    MAX_NUM_MATERIALS, MAX_NUM_TEXTURES,
};

#[repr(C, align(16))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ShaderData {
    pub num_textures: u32,
    pub num_materials: u32,
    pub num_lights: u32,
    pub light_data: [LightData; MAX_NUM_LIGHTS],
    pub textures_data: [ShaderTextureData; MAX_NUM_TEXTURES],
    pub materials_data: [ShaderMaterialData; MAX_NUM_MATERIALS],
}

impl Default for ShaderData {
    fn default() -> Self {
        Self {
            num_textures: 0,
            num_materials: 0,
            num_lights: 0,
            light_data: [LightData::default(); MAX_NUM_LIGHTS],
            textures_data: [ShaderTextureData::default(); MAX_NUM_TEXTURES],
            materials_data: [ShaderMaterialData::default(); MAX_NUM_MATERIALS],
        }
    }
}

impl ShaderData {
    #[allow(deref_nullptr)]
    pub fn debug_size() {
        let alignment_size = 16;
        let total_size = std::mem::size_of::<Self>();
        println!("UniformData info: Total size [{}]", total_size);

        let mut s = 0;
        print_field_size!(s, num_textures, u32, 1);
        print_field_size!(s, num_materials, u32, 1);
        print_field_size!(s, num_lights, u32, 1);
        print_field_size!(s, light_data, LightData, MAX_NUM_LIGHTS);
        print_field_size!(s, textures_data, ShaderTextureData, MAX_NUM_TEXTURES);
        print_field_size!(s, materials_data, ShaderMaterialData, MAX_NUM_MATERIALS);

        println!(
            "Alignment[{}] result: {} -> {}",
            alignment_size,
            if s == total_size && s % alignment_size == 0 {
                "OK"
            } else {
                "TO ALIGN"
            },
            (s as f32 / alignment_size as f32).ceil() as usize * alignment_size
        );

        LightData::debug_size(alignment_size);
        ShaderTextureData::debug_size(alignment_size);
        ShaderMaterialData::debug_size(alignment_size);
    }
}
