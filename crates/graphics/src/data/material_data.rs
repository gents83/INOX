use std::path::PathBuf;

use nrg_math::Vector4;
use nrg_serialize::{Deserialize, Serialize, SerializeFile};

use crate::TextureType;
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "nrg_serialize")]
pub enum MaterialAlphaMode {
    Opaque = 0,
    Mask = 1,
    Blend = 2,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct MaterialData {
    pub pipeline: PathBuf,
    pub textures: [PathBuf; TextureType::Count as _],
    pub texcoords_set: [usize; TextureType::Count as _],
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: MaterialAlphaMode,
    pub base_color: Vector4,
    pub emissive_color: Vector4,
    pub diffuse_color: Vector4,
    pub specular_color: Vector4,
}

impl SerializeFile for MaterialData {
    fn extension() -> &'static str {
        "material_data"
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
            base_color: Vector4::new(1., 1., 1., 1.),
            emissive_color: Vector4::new(1., 1., 1., 1.),
            diffuse_color: Vector4::new(1., 1., 1., 1.),
            specular_color: Vector4::new(0., 0., 0., 1.),
        }
    }
}
