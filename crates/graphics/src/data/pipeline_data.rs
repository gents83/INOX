use std::path::PathBuf;

use sabi_filesystem::convert_from_local_path;
use sabi_math::Matrix4;
use sabi_resources::DATA_FOLDER;
use sabi_serialize::{Deserialize, Serialize, SerializeFile};

use crate::{LightData, ShaderMaterialData, ShaderTextureData, TextureAtlas};

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
    Both,
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

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum DrawMode {
    Batch,
    Single,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(crate = "sabi_serialize")]
pub enum PipelineType {
    Custom,
    Default,
    Wireframe,
    UI,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct PipelineData {
    pub pipeline_type: PipelineType,
    pub fragment_shader: PathBuf,
    pub vertex_shader: PathBuf,
    pub tcs_shader: PathBuf,
    pub tes_shader: PathBuf,
    pub geometry_shader: PathBuf,
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
        "pipeline_data"
    }
}

impl Default for PipelineData {
    fn default() -> Self {
        Self {
            pipeline_type: PipelineType::Custom,
            fragment_shader: PathBuf::new(),
            vertex_shader: PathBuf::new(),
            tcs_shader: PathBuf::new(),
            tes_shader: PathBuf::new(),
            geometry_shader: PathBuf::new(),
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
        let data_path = PathBuf::from(DATA_FOLDER);
        if !self.vertex_shader.to_str().unwrap().is_empty() {
            self.vertex_shader =
                convert_from_local_path(data_path.as_path(), self.vertex_shader.as_path());
        }
        if !self.fragment_shader.to_str().unwrap().is_empty() {
            self.fragment_shader =
                convert_from_local_path(data_path.as_path(), self.fragment_shader.as_path());
        }
        if !self.tcs_shader.to_str().unwrap().is_empty() {
            self.tcs_shader =
                convert_from_local_path(data_path.as_path(), self.tcs_shader.as_path());
        }
        if !self.tes_shader.to_str().unwrap().is_empty() {
            self.tes_shader =
                convert_from_local_path(data_path.as_path(), self.tes_shader.as_path());
        }
        if !self.geometry_shader.to_str().unwrap().is_empty() {
            self.geometry_shader =
                convert_from_local_path(data_path.as_path(), self.geometry_shader.as_path());
        }
        self
    }
    pub fn has_same_shaders(&self, other: &PipelineData) -> bool {
        self.vertex_shader == other.vertex_shader
            && self.fragment_shader == other.fragment_shader
            && self.tcs_shader == other.tcs_shader
            && self.tes_shader == other.tes_shader
            && self.geometry_shader == other.geometry_shader
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
