use std::path::PathBuf;

use inox_bitmask::bitmask;
use inox_math::{Vector3, Vector4};
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::TextureType;

#[bitmask]
#[repr(u32)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(crate = "inox_serialize")]
pub enum MaterialFlags {
    None = 0,
    Unlit = 1,
    Iridescence = 1 << 1,
    Anisotropy = 1 << 2,
    Clearcoat = 1 << 3,
    Sheen = 1 << 4,
    Transmission = 1 << 5,
    Volume = 1 << 6,
    EmissiveStrength = 1 << 7,
    MetallicRoughness = 1 << 8,
    Specular = 1 << 9,
    SpecularGlossiness = 1 << 10,
    Ior = 1 << 11,
    AlphaModeOpaque = 1 << 12,
    AlphaModeMask = 1 << 13,
    AlphaModeBlend = 1 << 14,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MaterialData {
    pub textures: [PathBuf; TextureType::Count as _],
    pub texcoords_set: [usize; TextureType::Count as _],
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub ior: f32,
    pub transmission_factor: f32,
    pub base_color: Vector4,
    pub emissive_color: Vector3,
    pub emissive_strength: f32,
    pub diffuse_factor: Vector4,
    pub specular_glossiness_factor: Vector4,
    pub specular_factors: Vector4,
    pub attenuation_color_and_distance: Vector4,
    pub thickness_factor: f32,
    pub occlusion_strength: f32,
    pub alpha_cutoff: f32,
    pub flags: MaterialFlags,
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
            roughness_factor: 1.0,
            metallic_factor: 1.0,
            ior: 1.5,
            transmission_factor: 0.,
            alpha_cutoff: 1.,
            emissive_strength: 1.,
            base_color: Vector4::new(1., 1., 1., 1.),
            emissive_color: Vector3::new(1., 1., 1.),
            occlusion_strength: 0.,
            diffuse_factor: Vector4::new(1., 1., 1., 1.),
            specular_glossiness_factor: Vector4::new(0., 0., 0., 1.),
            specular_factors: Vector4::new(1., 1., 1., 0.),
            thickness_factor: 0.,
            attenuation_color_and_distance: Vector4::new(1., 1., 1., 0.),
            flags: MaterialFlags::Unlit,
        }
    }
}
