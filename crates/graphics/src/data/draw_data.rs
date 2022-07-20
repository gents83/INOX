use inox_bitmask::bitmask;
use inox_math::{MatBase, Matrix4};
use inox_serialize::{Deserialize, Serialize};

use crate::{
    MaterialAlphaMode, TextureType, VertexBufferLayoutBuilder, VertexFormat, INVALID_INDEX,
    MAX_TEXTURE_COORDS_SETS,
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

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawIndexedCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub base_index: u32,
    pub vertex_offset: i32,
    pub base_instance: u32,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub base_vertex: u32,
    pub base_instance: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawMesh {
    pub vertex_offset: u32,
    pub indices_offset: u32,
    pub meshlet_offset: u32,
    pub meshlet_count: u32,
    pub material_index: i32,
    pub mesh_flags: u32,
    padding: [u32; 2],
    pub matrix: [[f32; 4]; 4],
}

impl Default for DrawMesh {
    fn default() -> Self {
        Self {
            vertex_offset: 0,
            indices_offset: 0,
            meshlet_offset: 0,
            meshlet_count: 0,
            material_index: INVALID_INDEX,
            mesh_flags: 0,
            padding: [0u32; 2],
            matrix: Matrix4::default_identity().into(),
        }
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawMeshlet {
    pub mesh_index: u32,
    pub vertex_offset: u32,
    pub indices_offset: u32,
    pub indices_count: u32,
    pub center_radius: [f32; 4],
    pub cone_axis_cutoff: [f32; 4],
}

impl DrawMeshlet {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::instance();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<[f32; 4]>(VertexFormat::Float32x4.into());
        layout_builder.add_attribute::<[f32; 4]>(VertexFormat::Float32x4.into());
        layout_builder
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawMaterial {
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

impl Default for DrawMaterial {
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

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawVertex {
    pub position_and_color_offset: u32,
    pub normal_offset: i32,
    pub tangent_offset: i32,
    pub mesh_index: u32,
    pub uv_offset: [i32; MAX_TEXTURE_COORDS_SETS],
}

impl Default for DrawVertex {
    fn default() -> Self {
        Self {
            position_and_color_offset: 0,
            normal_offset: INVALID_INDEX,
            tangent_offset: INVALID_INDEX,
            mesh_index: 0,
            uv_offset: [INVALID_INDEX; MAX_TEXTURE_COORDS_SETS],
        }
    }
}

impl DrawVertex {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<i32>(VertexFormat::Sint32.into());
        layout_builder.add_attribute::<i32>(VertexFormat::Sint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder
            .add_attribute::<[i32; MAX_TEXTURE_COORDS_SETS]>(VertexFormat::Sint32x4.into());
        layout_builder
    }
}
