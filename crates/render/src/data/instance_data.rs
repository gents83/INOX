use inox_math::Vector4;

use crate::{VertexBufferLayoutBuilder, INVALID_INDEX};

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct InstanceData {
    pub draw_area: [f32; 4],
    pub matrix: [[f32; 4]; 4],
    pub material_index: i32,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            draw_area: [0.; 4],
            matrix: [[0.; 4]; 4],
            material_index: INVALID_INDEX,
        }
    }
}

impl InstanceData {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::instance();
        layout_builder.starting_location(starting_location);
        //draw_area
        layout_builder.add_attribute::<Vector4>(wgpu::VertexFormat::Float32x4);
        //matrix_0
        layout_builder.add_attribute::<Vector4>(wgpu::VertexFormat::Float32x4);
        //matrix_1
        layout_builder.add_attribute::<Vector4>(wgpu::VertexFormat::Float32x4);
        //matrix_2
        layout_builder.add_attribute::<Vector4>(wgpu::VertexFormat::Float32x4);
        //matrix_3
        layout_builder.add_attribute::<Vector4>(wgpu::VertexFormat::Float32x4);
        //material_index
        layout_builder.add_attribute::<i32>(wgpu::VertexFormat::Sint32);
        layout_builder
    }
}
