use inox_math::{Vector3, Vector4};

use crate::{VertexBufferLayoutBuilder, INVALID_INDEX};

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct InstanceData {
    pub id: u32,
    pub draw_area: [f32; 4],
    pub matrix: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 3]; 3],
    pub material_index: i32,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            id: 0,
            draw_area: [0.; 4],
            matrix: [[0.; 4]; 4],
            normal_matrix: [[0.; 3]; 3],
            material_index: INVALID_INDEX,
        }
    }
}

impl InstanceData {
    pub fn descriptor<'a>() -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::instance();
        layout_builder.starting_location(8);
        //id
        layout_builder.add_attribute::<u32>(wgpu::VertexFormat::Uint32);
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
        //normal_matrix_0
        layout_builder.add_attribute::<Vector3>(wgpu::VertexFormat::Float32x3);
        //normal_matrix_1
        layout_builder.add_attribute::<Vector3>(wgpu::VertexFormat::Float32x3);
        //normal_matrix_2
        layout_builder.add_attribute::<Vector3>(wgpu::VertexFormat::Float32x3);
        //material_index
        layout_builder.add_attribute::<i32>(wgpu::VertexFormat::Sint32);
        layout_builder
    }
}
