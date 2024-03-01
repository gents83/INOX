use std::path::PathBuf;

use inox_filesystem::convert_from_local_path;

use inox_resources::Data;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::MeshFlags;

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

impl From<PolygonMode> for wgpu::PolygonMode {
    fn from(mode: PolygonMode) -> Self {
        match mode {
            PolygonMode::Fill => wgpu::PolygonMode::Fill,
            PolygonMode::Line => wgpu::PolygonMode::Line,
            PolygonMode::Point => wgpu::PolygonMode::Point,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum FrontFace {
    CounterClockwise,
    Clockwise,
}

impl From<FrontFace> for wgpu::FrontFace {
    fn from(mode: FrontFace) -> Self {
        match mode {
            FrontFace::CounterClockwise => wgpu::FrontFace::Ccw,
            FrontFace::Clockwise => wgpu::FrontFace::Cw,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum CullMode {
    None,
    Back,
    Front,
}

impl From<CullMode> for Option<wgpu::Face> {
    fn from(mode: CullMode) -> Self {
        match mode {
            CullMode::Back => Some(wgpu::Face::Back),
            CullMode::Front => Some(wgpu::Face::Front),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
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

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum CompareFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl From<CompareFunction> for wgpu::CompareFunction {
    fn from(f: CompareFunction) -> Self {
        match f {
            CompareFunction::Never => wgpu::CompareFunction::Never,
            CompareFunction::Less => wgpu::CompareFunction::Less,
            CompareFunction::Equal => wgpu::CompareFunction::Equal,
            CompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
            CompareFunction::Greater => wgpu::CompareFunction::Greater,
            CompareFunction::NotEqual => wgpu::CompareFunction::NotEqual,
            CompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            CompareFunction::Always => wgpu::CompareFunction::Always,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum BlendOperation {
    /// Src + Dst
    Add = 0,
    /// Src - Dst
    Subtract = 1,
    /// Dst - Src
    ReverseSubtract = 2,
    /// min(Src, Dst)
    Min = 3,
    /// max(Src, Dst)
    Max = 4,
}

impl From<BlendOperation> for wgpu::BlendOperation {
    fn from(c: BlendOperation) -> Self {
        match c {
            BlendOperation::Add => wgpu::BlendOperation::Add,
            BlendOperation::Subtract => wgpu::BlendOperation::Subtract,
            BlendOperation::ReverseSubtract => wgpu::BlendOperation::ReverseSubtract,
            BlendOperation::Min => wgpu::BlendOperation::Min,
            BlendOperation::Max => wgpu::BlendOperation::Max,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Eq, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum DrawMode {
    Batch,
    Single,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct RenderPipelineData {
    pub vertex_shader: PathBuf,
    pub fragment_shader: PathBuf,
    pub front_face: FrontFace,
    pub culling: CullMode,
    pub mode: PolygonMode,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_operation: BlendOperation,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_operation: BlendOperation,
    pub sampling_count: u32,
    pub mesh_flags: MeshFlags,
}

impl SerializeFile for RenderPipelineData {
    fn extension() -> &'static str {
        "render_pipeline"
    }
}

impl Default for RenderPipelineData {
    fn default() -> Self {
        Self {
            vertex_shader: PathBuf::new(),
            fragment_shader: PathBuf::new(),
            front_face: FrontFace::CounterClockwise,
            culling: CullMode::Back,
            mode: PolygonMode::Fill,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            src_color_blend_factor: BlendFactor::One,
            dst_color_blend_factor: BlendFactor::OneMinusSrcColor,
            color_blend_operation: BlendOperation::Add,
            src_alpha_blend_factor: BlendFactor::One,
            dst_alpha_blend_factor: BlendFactor::OneMinusSrcAlpha,
            alpha_blend_operation: BlendOperation::Add,
            sampling_count: 1,
            mesh_flags: MeshFlags::Visible | MeshFlags::Opaque,
        }
    }
}

impl RenderPipelineData {
    pub fn canonicalize_paths(&self) -> Self {
        let mut data = self.clone();
        let data_path = Data::platform_data_folder();
        if !data.vertex_shader.to_str().unwrap().is_empty() {
            data.vertex_shader =
                convert_from_local_path(data_path.as_path(), data.vertex_shader.as_path());
        }
        if !data.fragment_shader.to_str().unwrap().is_empty() {
            data.fragment_shader =
                convert_from_local_path(data_path.as_path(), data.fragment_shader.as_path());
        }
        data
    }
    pub fn has_same_shaders(&self, other: &RenderPipelineData) -> bool {
        self.vertex_shader == other.vertex_shader && self.fragment_shader == other.fragment_shader
    }
}
