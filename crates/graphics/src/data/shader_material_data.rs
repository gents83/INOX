use crate::{print_field_size, MaterialAlphaMode, TextureType, INVALID_INDEX};

pub const MAX_NUM_MATERIALS: usize = 512;

#[repr(C, align(16))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ShaderMaterialData {
    pub textures_indices: [i32; TextureType::Count as _],
    pub textures_coord_set: [u32; TextureType::Count as _],
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: u32,
    pub base_color: [f32; 4],
    pub emissive_color: [f32; 4],
    pub diffuse_color: [f32; 4],
    pub specular_color: [f32; 4],
}

impl Default for ShaderMaterialData {
    fn default() -> Self {
        Self {
            textures_indices: [INVALID_INDEX as _; TextureType::Count as _],
            textures_coord_set: [0; TextureType::Count as _],
            roughness_factor: 1.,
            metallic_factor: 1.,
            alpha_cutoff: 1.,
            alpha_mode: MaterialAlphaMode::Opaque as _,
            base_color: [1., 1., 1., 1.],
            emissive_color: [1., 1., 1., 1.],
            diffuse_color: [1., 1., 1., 1.],
            specular_color: [0., 0., 0., 1.],
        }
    }
}

impl ShaderMaterialData {
    #[allow(deref_nullptr)]
    pub fn debug_size(alignment_size: usize) {
        let total_size = std::mem::size_of::<Self>();
        println!("ShaderMaterialData info: Total size [{}]", total_size);

        let mut s = 0;
        print_field_size!(s, textures_indices, [i32; TextureType::Count as _], 1);
        print_field_size!(s, textures_coord_set, [u32; TextureType::Count as _], 1);
        print_field_size!(s, roughness_factor, f32, 1);
        print_field_size!(s, metallic_factor, f32, 1);
        print_field_size!(s, alpha_cutoff, f32, 1);
        print_field_size!(s, alpha_mode, u32, 1);
        print_field_size!(s, base_color, [f32; 4], 1);
        print_field_size!(s, emissive_color, [f32; 4], 1);
        print_field_size!(s, diffuse_color, [f32; 4], 1);
        print_field_size!(s, specular_color, [f32; 4], 1);

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
