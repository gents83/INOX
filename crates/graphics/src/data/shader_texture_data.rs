use crate::print_field_size;

pub const MAX_NUM_TEXTURES: usize = 512;

#[repr(C, align(16))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ShaderTextureData {
    pub texture_index: u32,
    pub layer_index: u32,
    pub total_width: f32,
    pub total_height: f32,
    pub area: [f32; 4],
}

impl Default for ShaderTextureData {
    fn default() -> Self {
        Self {
            texture_index: 0,
            layer_index: 0,
            total_width: 0.,
            total_height: 0.,
            area: [0., 0., 1., 1.],
        }
    }
}

impl ShaderTextureData {
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

impl ShaderTextureData {
    #[allow(deref_nullptr)]
    pub fn debug_size(alignment_size: usize) {
        let total_size = std::mem::size_of::<Self>();
        println!("ShaderTextureData info: Total size [{}]", total_size,);

        let mut s = 0;
        print_field_size!(s, texture_index, u32, 1);
        print_field_size!(s, layer_index, u32, 1);
        print_field_size!(s, total_width, f32, 1);
        print_field_size!(s, total_height, f32, 1);
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
