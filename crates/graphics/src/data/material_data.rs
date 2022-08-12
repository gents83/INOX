use std::path::PathBuf;

use inox_math::{Vector3, Vector4};
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::TextureType;

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(crate = "inox_serialize")]
pub enum MaterialAlphaMode {
    Opaque = 0,
    Mask = 1,
    Blend = 2,
}

impl From<MaterialAlphaMode> for u32 {
    fn from(val: MaterialAlphaMode) -> Self {
        match val {
            MaterialAlphaMode::Opaque => 0,
            MaterialAlphaMode::Mask => 1,
            MaterialAlphaMode::Blend => 2,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MaterialData {
    pub textures: [PathBuf; TextureType::Count as _],
    pub texcoords_set: [usize; TextureType::Count as _],
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: MaterialAlphaMode,
    pub base_color: Vector4,
    pub emissive_color: Vector3,
    pub occlusion_strength: f32,
    pub diffuse_color: Vector4,
    pub specular_color: Vector4,
}

impl SerializeFile for MaterialData {
    fn extension() -> &'static str {
        "material"
    }
}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            textures: Default::default(),
            texcoords_set: Default::default(),
            roughness_factor: 1.,
            metallic_factor: 1.,
            alpha_cutoff: 1.,
            alpha_mode: MaterialAlphaMode::Opaque,
            base_color: Vector4::new(1., 1., 1., 1.),
            emissive_color: Vector3::new(1., 1., 1.),
            occlusion_strength: 0.,
            diffuse_color: Vector4::new(1., 1., 1., 1.),
            specular_color: Vector4::new(0., 0., 0., 1.),
        }
    }
}
