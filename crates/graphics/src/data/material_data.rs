use std::path::PathBuf;

use sabi_serialize::*;

use crate::TextureType;
#[derive(Serializable, Debug, PartialEq, Clone, Copy)]
pub enum MaterialAlphaMode {
    Opaque = 0,
    Mask = 1,
    Blend = 2,
}

#[derive(Serializable, Debug, PartialEq, Clone)]
pub struct MaterialData {
    pub pipeline: PathBuf,
    pub textures: [PathBuf; TextureType::Count as _],
    pub texcoords_set: [usize; TextureType::Count as _],
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: MaterialAlphaMode,
    pub base_color: [f32; 4],
    pub emissive_color: [f32; 4],
    pub diffuse_color: [f32; 4],
    pub specular_color: [f32; 4],
}

impl SerializeFile for MaterialData {
    fn extension() -> &'static str {
        "material"
    }
}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            pipeline: PathBuf::new(),
            textures: Default::default(),
            texcoords_set: Default::default(),
            roughness_factor: 1.,
            metallic_factor: 1.,
            alpha_cutoff: 1.,
            alpha_mode: MaterialAlphaMode::Opaque,
            base_color: [1.; 4],
            emissive_color: [1.; 4],
            diffuse_color: [1.; 4],
            specular_color: [0., 0., 0., 1.],
        }
    }
}
