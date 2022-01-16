use std::path::PathBuf;

use sabi_filesystem::convert_from_local_path;
use sabi_math::Matrix4;
use sabi_resources::Data;
use sabi_serialize::{generate_uid_from_string, Deserialize, Serialize, SerializeFile, Uid};

use crate::{LightData, ShaderMaterialData, ShaderTextureData, TextureAtlas};

pub const DEFAULT_PIPELINE_IDENTIFIER: &str = "SABI_Default_Pipeline";
pub const WIREFRAME_PIPELINE_IDENTIFIER: &str = "EditorWireframe";

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct PipelineIdentifier(Uid);

impl PipelineIdentifier {
    pub fn new(string: &str) -> Self {
        Self(generate_uid_from_string(string))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum PolygonModeType {
    Fill,
    Line,
    Point,
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum CullingModeType {
    None,
    Back,
    Front,
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SrcAlphaSaturate,
}

impl From<BlendFactor> for wgpu::BlendFactor {
    fn from(blend_factor: BlendFactor) -> Self {
        match blend_factor {
            BlendFactor::Zero => wgpu::BlendFactor::Zero,
            BlendFactor::One => wgpu::BlendFactor::One,
            BlendFactor::SrcColor => wgpu::BlendFactor::Src,
            BlendFactor::OneMinusSrcColor => wgpu::BlendFactor::OneMinusSrc,
            BlendFactor::DstColor => wgpu::BlendFactor::Dst,
            BlendFactor::OneMinusDstColor => wgpu::BlendFactor::OneMinusDst,
            BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
            BlendFactor::DstAlpha => wgpu::BlendFactor::DstAlpha,
            BlendFactor::OneMinusDstAlpha => wgpu::BlendFactor::OneMinusDstAlpha,
            BlendFactor::ConstantColor => wgpu::BlendFactor::Constant,
            BlendFactor::OneMinusConstantColor => wgpu::BlendFactor::OneMinusConstant,
            BlendFactor::ConstantAlpha => wgpu::BlendFactor::Constant,
            BlendFactor::OneMinusConstantAlpha => wgpu::BlendFactor::OneMinusConstant,
            BlendFactor::SrcAlphaSaturate => wgpu::BlendFactor::SrcAlphaSaturated,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum DrawMode {
    Batch,
    Single,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct PipelineData {
    pub identifier: String,
    pub shader: PathBuf,
    pub culling: CullingModeType,
    pub mode: PolygonModeType,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub draw_mode: DrawMode,
}

impl SerializeFile for PipelineData {
    fn extension() -> &'static str {
        "pipeline"
    }
}

impl Default for PipelineData {
    fn default() -> Self {
        Self {
            identifier: DEFAULT_PIPELINE_IDENTIFIER.to_string(),
            shader: PathBuf::new(),
            culling: CullingModeType::Back,
            mode: PolygonModeType::Fill,
            src_color_blend_factor: BlendFactor::One,
            dst_color_blend_factor: BlendFactor::OneMinusSrcColor,
            src_alpha_blend_factor: BlendFactor::One,
            dst_alpha_blend_factor: BlendFactor::OneMinusSrcAlpha,
            draw_mode: DrawMode::Batch,
        }
    }
}

impl PipelineData {
    pub fn canonicalize_paths(mut self) -> Self {
        let data_path = Data::data_folder();
        if !self.shader.to_str().unwrap().is_empty() {
            self.shader = convert_from_local_path(data_path.as_path(), self.shader.as_path());
        }
        self
    }
    pub fn has_same_shaders(&self, other: &PipelineData) -> bool {
        self.shader == other.shader
    }
}

pub struct PipelineBindingData<'a> {
    pub width: u32,
    pub height: u32,
    pub view: &'a Matrix4,
    pub proj: &'a Matrix4,
    pub textures: &'a [TextureAtlas],
    pub used_textures: &'a [bool],
    pub light_data: &'a [LightData],
    pub texture_data: &'a [ShaderTextureData],
    pub material_data: &'a [ShaderMaterialData],
}
