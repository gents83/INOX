use inox_bitmask::bitmask;
use inox_math::{Mat4Ops, Matrix4};

use crate::{
    MaterialAlphaMode, TextureType, VertexBufferLayoutBuilder, VertexFormat, INVALID_INDEX,
};

// Pipeline has a list of meshes to process
// Meshes can switch pipeline at runtime
// Material doesn't know pipeline anymore
// Material is now generic data for several purposes

#[bitmask]
pub enum DrawCommandType {
    PerMeshlet,
    PerTriangle,
}

#[repr(C)]
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct DrawIndexedCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub base_index: u32,
    pub vertex_offset: i32,
    pub base_instance: u32,
}

#[repr(C, align(4))]
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct DrawCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub base_vertex: u32,
    pub base_instance: u32,
}

#[repr(C, align(4))]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct GPUMesh {
    pub vertices_position_offset: u32,
    pub vertices_attribute_offset: u32,
    pub vertices_attribute_layout: u32,
    pub material_index: i32,
    pub orientation: [f32; 4],
    pub position: [f32; 3],
    pub meshlets_offset: u32,
    pub scale: [f32; 3],
    pub blas_index: u32,
}

impl Default for GPUMesh {
    fn default() -> Self {
        Self {
            vertices_position_offset: 0,
            vertices_attribute_offset: 0,
            material_index: INVALID_INDEX,
            blas_index: 0,
            position: [0.; 3],
            meshlets_offset: 0,
            scale: [1.; 3],
            vertices_attribute_layout: 0,
            orientation: [0., 0., 0., 1.],
        }
    }
}

impl GPUMesh {
    pub fn transform(&self) -> Matrix4 {
        Matrix4::from_translation_orientation_scale(
            self.position.into(),
            self.orientation.into(),
            self.scale.into(),
        )
    }
}

#[repr(C, align(4))]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct ConeCulling {
    pub center: [f32; 3],
    pub cone_axis_cutoff: [i8; 4],
}

#[repr(C, align(4))]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct GPUMeshlet {
    pub mesh_index: u32,
    pub indices_offset: u32,
    pub indices_count: u32,
    pub blas_index: u32,
}

impl GPUMeshlet {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::instance();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder
    }
}

#[repr(C, align(4))]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct GPUBHVNode {
    pub min: [f32; 3],
    pub miss: i32,
    pub max: [f32; 3],
    pub reference: i32,
}

#[repr(C, align(4))]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct GPUMaterial {
    pub textures_indices: [i32; TextureType::Count as _],
    pub textures_coord_set: [u32; TextureType::Count as _],
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub alpha_cutoff: f32,
    pub alpha_mode: u32,
    pub base_color: [f32; 4],
    pub emissive_color: [f32; 3],
    pub occlusion_strength: f32,
    pub diffuse_color: [f32; 4],
    pub specular_color: [f32; 4],
}

impl Default for GPUMaterial {
    fn default() -> Self {
        Self {
            textures_indices: [INVALID_INDEX; TextureType::Count as _],
            textures_coord_set: [0; TextureType::Count as _],
            roughness_factor: 0.,
            metallic_factor: 0.,
            alpha_cutoff: 1.,
            alpha_mode: MaterialAlphaMode::Opaque.into(),
            base_color: [1.; 4],
            emissive_color: [1.; 3],
            occlusion_strength: 0.0,
            diffuse_color: [1.; 4],
            specular_color: [1.; 4],
        }
    }
}

#[repr(C, align(4))]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct GPURay {
    pub origin: [f32; 3],
    pub t_min: f32,
    pub direction: [f32; 3],
    pub t_max: f32,
}

#[repr(C, align(4))]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct GPURuntimeVertexData {
    pub world_pos: [f32; 3],
    pub mesh_index: u32,
}

impl GPURuntimeVertexData {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<[f32; 3]>(VertexFormat::Float32x3.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder
    }
}
