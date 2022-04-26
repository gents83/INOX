use std::mem::size_of;

use inox_math::{VecBase, Vector2, Vector3, Vector4};
use inox_serialize::{Deserialize, Serialize};

pub const MAX_TEXTURE_COORDS_SETS: usize = 4;
pub type VertexFormatBits = u32;

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
#[rustfmt::skip]
pub enum VertexFormat {
    PositionF32x2   = 0,
    PositionF32x3   = 1,
    NormalF32x3     = 2,
    TangentF32x4    = 3,
    ColorF32x4      = 4,
    ColorU32        = 5,
    TexCoord0F32x2  = 6,
    TexCoord1F32x2  = 7,
    TexCoord2F32x2  = 8,
    TexCoord3F32x2  = 9,
    Count           = 10,
}

impl VertexFormat {
    pub fn to_bits(attributes: &[Self]) -> VertexFormatBits {
        let mut value = 0;
        attributes.iter().for_each(|a| {
            value |= 1 << a.clone() as u32;
        });
        value
    }
    pub fn ui() -> Vec<Self> {
        vec![
            VertexFormat::PositionF32x2,
            VertexFormat::TexCoord0F32x2,
            VertexFormat::ColorU32,
        ]
    }
    pub fn pbr() -> Vec<Self> {
        vec![
            VertexFormat::PositionF32x3,
            VertexFormat::NormalF32x3,
            VertexFormat::ColorF32x4,
            VertexFormat::TexCoord0F32x2,
            VertexFormat::TexCoord1F32x2,
            VertexFormat::TexCoord2F32x2,
            VertexFormat::TexCoord3F32x2,
        ]
    }
    pub fn size(&self) -> usize {
        match self {
            VertexFormat::PositionF32x2 => size_of::<Vector2>(),
            VertexFormat::PositionF32x3 => size_of::<Vector3>(),
            VertexFormat::NormalF32x3 => size_of::<Vector3>(),
            VertexFormat::TangentF32x4 => size_of::<Vector4>(),
            VertexFormat::ColorF32x4 => size_of::<Vector4>(),
            VertexFormat::ColorU32 => size_of::<u32>(),
            VertexFormat::TexCoord0F32x2 => size_of::<Vector2>(),
            VertexFormat::TexCoord1F32x2 => size_of::<Vector2>(),
            VertexFormat::TexCoord2F32x2 => size_of::<Vector2>(),
            VertexFormat::TexCoord3F32x2 => size_of::<Vector2>(),
            VertexFormat::Count => 0,
        }
    }
    pub fn format(&self) -> wgpu::VertexFormat {
        match self {
            VertexFormat::PositionF32x2 => wgpu::VertexFormat::Float32x2,
            VertexFormat::PositionF32x3 => wgpu::VertexFormat::Float32x3,
            VertexFormat::NormalF32x3 => wgpu::VertexFormat::Float32x3,
            VertexFormat::TangentF32x4 => wgpu::VertexFormat::Float32x4,
            VertexFormat::ColorF32x4 => wgpu::VertexFormat::Float32x4,
            VertexFormat::ColorU32 => wgpu::VertexFormat::Uint32,
            VertexFormat::TexCoord0F32x2 => wgpu::VertexFormat::Float32x2,
            VertexFormat::TexCoord1F32x2 => wgpu::VertexFormat::Float32x2,
            VertexFormat::TexCoord2F32x2 => wgpu::VertexFormat::Float32x2,
            VertexFormat::TexCoord3F32x2 => wgpu::VertexFormat::Float32x2,
            VertexFormat::Count => panic!("Invalid attribute with no format"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "inox_serialize")]
pub struct PbrVertexData {
    pub pos: Vector3,
    pub normal: Vector3,
    pub color: Vector4,
    pub tex_coord: [Vector2; MAX_TEXTURE_COORDS_SETS],
}

impl Default for PbrVertexData {
    fn default() -> PbrVertexData {
        PbrVertexData {
            pos: Vector3::default_zero(),
            normal: Vector3::new(0., 0., 1.),
            color: Vector4::new(1., 1., 1., 1.),
            tex_coord: [Vector2::default_zero(); MAX_TEXTURE_COORDS_SETS],
        }
    }
}

pub struct VertexBufferLayoutBuilder<'a> {
    layout: wgpu::VertexBufferLayout<'a>,
    attributes: Vec<wgpu::VertexAttribute>,
    offset: wgpu::BufferAddress,
    location: u32,
}

impl<'a> VertexBufferLayoutBuilder<'a> {
    pub fn create_from_vertex_data_attribute(vertex_data_attribute: &'a [VertexFormat]) -> Self {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        vertex_data_attribute.iter().for_each(|attribute| {
            layout_builder.attributes.push(wgpu::VertexAttribute {
                offset: layout_builder.offset,
                shader_location: layout_builder.location,
                format: attribute.format(),
            });
            layout_builder.offset += attribute.size() as wgpu::BufferAddress;
            layout_builder.location += 1;
        });
        layout_builder.layout.array_stride = layout_builder.offset;
        layout_builder
    }

    pub fn vertex() -> Self {
        Self {
            attributes: vec![],
            layout: wgpu::VertexBufferLayout {
                attributes: &[],
                array_stride: 0,
                step_mode: wgpu::VertexStepMode::Vertex,
            },
            offset: 0,
            location: 0,
        }
    }
    pub fn instance() -> Self {
        Self {
            attributes: vec![],
            layout: wgpu::VertexBufferLayout {
                attributes: &[],
                array_stride: 0,
                step_mode: wgpu::VertexStepMode::Instance,
            },
            offset: 0,
            location: 0,
        }
    }
    pub fn add_attribute<T>(&mut self, format: wgpu::VertexFormat) {
        self.attributes.push(wgpu::VertexAttribute {
            offset: self.offset,
            shader_location: self.location,
            format,
        });
        self.offset += std::mem::size_of::<T>() as wgpu::BufferAddress;
        self.location += 1;
    }

    pub fn starting_location(&mut self, location: u32) {
        self.location = location;
    }

    pub fn location(&self) -> u32 {
        self.location
    }

    pub fn build(&'a self) -> wgpu::VertexBufferLayout<'a> {
        let mut layout = self.layout.clone();
        layout.array_stride = self.offset;
        layout.attributes = &self.attributes;
        layout
    }
}
