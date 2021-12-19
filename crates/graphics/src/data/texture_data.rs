use sabi_serialize::*;

#[derive(Serializable, Debug, PartialEq, Clone)]
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
