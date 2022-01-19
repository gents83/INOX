use crate::print_field_size;
use sabi_serialize::{Deserialize, Serialize};

pub const MAX_NUM_TEXTURES: usize = 512;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum TextureType {
    BaseColor = 0,
    MetallicRoughness = 1,
    Normal = 2,
    Emissive = 3,
    Occlusion = 4,
    SpecularGlossiness = 5,
    Diffuse = 6,
    _EmptyForPadding = 7,
    Count = 8,
}

impl From<TextureType> for usize {
    fn from(val: TextureType) -> Self {
        val as _
    }
}
impl From<usize> for TextureType {
    fn from(value: usize) -> Self {
        match value {
            0 => TextureType::BaseColor,
            1 => TextureType::MetallicRoughness,
            2 => TextureType::Normal,
            3 => TextureType::Emissive,
            4 => TextureType::Occlusion,
            5 => TextureType::SpecularGlossiness,
            6 => TextureType::Diffuse,
            7 => TextureType::_EmptyForPadding,
            8 => TextureType::Count,
            _ => panic!("Invalid TextureType value: {}", value),
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TextureData {
    pub texture_index: u32,
    pub layer_index: u32,
    pub total_width: u32,
    pub total_height: u32,
    pub area: [f32; 4],
}

impl Default for TextureData {
    fn default() -> Self {
        Self {
            texture_index: 0,
            layer_index: 0,
            total_width: 0,
            total_height: 0,
            area: [0., 0., 1., 1.],
        }
    }
}

impl TextureData {
    pub fn get_texture_index(&self) -> u32 {
        self.texture_index
    }
    pub fn get_layer_index(&self) -> u32 {
        self.layer_index
    }
    pub fn get_width(&self) -> u32 {
        self.area[2] as _
    }
    pub fn get_height(&self) -> u32 {
        self.area[3] as _
    }
}

impl TextureData {
    #[allow(deref_nullptr)]
    pub fn debug_size(alignment_size: usize) {
        let total_size = std::mem::size_of::<Self>();
        println!("ShaderTextureData info: Total size [{}]", total_size,);

        let mut s = 0;
        print_field_size!(s, texture_index, u32, 1);
        print_field_size!(s, layer_index, u32, 1);
        print_field_size!(s, total_width, u32, 1);
        print_field_size!(s, total_height, u32, 1);
        print_field_size!(s, area, [f32; 4], 1);

        println!(
            "Alignment result: {} -> {}",
            if s == total_size && s % alignment_size == 0 {
                "OK"
            } else {
                "TO ALIGN"
            },
            (s as f32 / alignment_size as f32).ceil() as usize * alignment_size
        );
    }
}
