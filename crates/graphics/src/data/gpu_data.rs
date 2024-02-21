use inox_bitmask::bitmask;
use inox_math::{quantize_half, Mat4Ops, Matrix4};

use crate::{
    declare_as_dirty_binding, MaterialFlags, TextureType, VertexBufferLayoutBuilder, VertexFormat,
    INVALID_INDEX,
};

pub const MAX_LOD_LEVELS: usize = 8;
pub const MESHLETS_GROUP_SIZE: usize = 4;
pub const HALF_MESHLETS_GROUP_SIZE: usize = MESHLETS_GROUP_SIZE / 2;

// Pipeline has a list of meshes to process
// Meshes can switch pipeline at runtime
// Material doesn't know pipeline anymore
// Material is now generic data for several purposes

#[repr(C)]
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct DispatchCommandSize {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
declare_as_dirty_binding!(DispatchCommandSize);

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

#[repr(C)]
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct DrawCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub base_vertex: u32,
    pub base_instance: u32,
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct GPUMesh {
    pub vertices_position_offset: u32,
    pub vertices_attribute_offset: u32,
    pub flags_and_vertices_attribute_layout: u32, // 16 bits | 16 bits
    pub material_index: i32,
    pub orientation: [f32; 4],
    pub position: [f32; 3],
    pub meshlets_offset: u32,
    pub scale: [f32; 3],
    pub blas_index: u32,
    pub lods_meshlets_offset: [u32; MAX_LOD_LEVELS], // 16 bits start | 16 bits end
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
            flags_and_vertices_attribute_layout: 0,
            orientation: [0., 0., 0., 1.],
            lods_meshlets_offset: [0; MAX_LOD_LEVELS],
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

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct GPUMeshlet {
    pub mesh_index_and_lod_level: u32, // 29 mesh + 3 lod bits
    pub indices_offset: u32,
    pub indices_count: u32,
    pub bvh_offset: u32,
    pub child_meshlets: [i32; MESHLETS_GROUP_SIZE],
}

impl GPUMeshlet {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::instance();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<[i32; 4]>(VertexFormat::Uint32x4.into());
        layout_builder
    }
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct GPUMaterial {
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub ior: f32,
    pub transmission_factor: f32,
    pub base_color: [f32; 4],
    pub emissive_color: [f32; 3],
    pub emissive_strength: f32,
    pub diffuse_color: [f32; 4],
    pub specular_color: [f32; 4],
    pub specular_factors: [f32; 4],
    pub attenuation_color_and_distance: [f32; 4],
    pub thickness_factor: f32,
    pub normal_scale_and_alpha_cutoff: u32,
    pub occlusion_strength: f32,
    pub flags: u32,
    pub textures_index_and_coord_set: [u32; TextureType::Count as _],
}

impl Default for GPUMaterial {
    fn default() -> Self {
        Self {
            textures_index_and_coord_set: [0; TextureType::Count as _],
            roughness_factor: 1.0,
            metallic_factor: 1.0,
            ior: 1.5,
            transmission_factor: 0.,
            base_color: [1.; 4],
            emissive_color: [1.; 3],
            emissive_strength: 1.0,
            diffuse_color: [1.; 4],
            specular_color: [1.; 4],
            specular_factors: [1.; 4],
            attenuation_color_and_distance: [1., 1., 1., 0.],
            thickness_factor: 0.,
            normal_scale_and_alpha_cutoff: quantize_half(1.) as u32
                | (quantize_half(1.) as u32) << 16,
            occlusion_strength: 0.0,
            flags: MaterialFlags::Unlit.into(),
        }
    }
}

#[repr(C)]
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
