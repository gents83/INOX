use inox_math::{VecBase, Vector2, Vector3, Vector4};
use inox_serialize::{Deserialize, Serialize};

use crate::InstanceData;

pub const MAX_TEXTURE_COORDS_SETS: usize = 4;

#[repr(C, align(16))]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "inox_serialize")]
pub struct VertexData {
    pub pos: Vector3,
    pub normal: Vector3,
    pub tangent: Vector3,
    pub color: Vector4,
    pub tex_coord: [Vector2; MAX_TEXTURE_COORDS_SETS],
}

impl Default for VertexData {
    fn default() -> VertexData {
        VertexData {
            pos: Vector3::default_zero(),
            normal: Vector3::new(0., 0., 1.),
            tangent: Vector3::new(0., 0., 1.),
            color: Vector4::new(1., 1., 1., 1.),
            tex_coord: [Vector2::default_zero(); MAX_TEXTURE_COORDS_SETS],
        }
    }
}

impl VertexData {
    pub fn descriptor<'a>() -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        layout_builder.starting_location(0);
        //pos
        layout_builder.add_attribute::<Vector3>(wgpu::VertexFormat::Float32x3);
        //normal
        layout_builder.add_attribute::<Vector3>(wgpu::VertexFormat::Float32x3);
        //tangent
        layout_builder.add_attribute::<Vector3>(wgpu::VertexFormat::Float32x3);
        //color
        layout_builder.add_attribute::<Vector4>(wgpu::VertexFormat::Float32x4);
        (0..MAX_TEXTURE_COORDS_SETS).for_each(|_| {
            layout_builder.add_attribute::<Vector2>(wgpu::VertexFormat::Float32x2);
        });
        layout_builder
    }
}

pub struct VertexBufferLayoutBuilder<'a> {
    layout: wgpu::VertexBufferLayout<'a>,
    attributes: Vec<wgpu::VertexAttribute>,
    offset: wgpu::BufferAddress,
    location: u32,
}

impl<'a> VertexBufferLayoutBuilder<'a> {
    pub fn vertex() -> Self {
        Self {
            attributes: vec![],
            layout: wgpu::VertexBufferLayout {
                attributes: &[],
                array_stride: std::mem::size_of::<VertexData>() as wgpu::BufferAddress,
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
                array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
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

    pub fn build(&'a self) -> wgpu::VertexBufferLayout<'a> {
        let mut layout = self.layout.clone();
        layout.attributes = &self.attributes;
        layout
    }
}
