use crate::colors::ColorType;


pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub channel_count: u8,
    pub color_type: ColorType,
}

impl Default for Image {
    fn default() -> Self {
        Image {
            data: Vec::new(),
            width: 0,
            height: 0,
            channel_count: 0,
            color_type: ColorType::Rgba8,
        }
    }
}
