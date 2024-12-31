use inox_bitmask::bitmask;
use inox_math::quantize_half;

use crate::{
    AsBinding, BufferRef, MaterialFlags, RenderContext, TextureType, VertexBufferLayoutBuilder,
    VertexFormat, INVALID_INDEX,
};

pub const MAX_LOD_LEVELS: usize = 8;

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
    pub meshlets_offset: u32,
    pub blas_index: u32,
    pub indices_offset: u32,
    pub indices_count: u32,
    pub lods_meshlets_offset: [u32; MAX_LOD_LEVELS], // 16 bits start | 16 bits end
}

impl Default for GPUMesh {
    fn default() -> Self {
        Self {
            vertices_position_offset: 0,
            vertices_attribute_offset: 0,
            meshlets_offset: 0,
            material_index: INVALID_INDEX,
            flags_and_vertices_attribute_layout: 0,
            blas_index: 0,
            indices_offset: 0,
            indices_count: 0,
            lods_meshlets_offset: [0; MAX_LOD_LEVELS],
        }
    }
}

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct GPUMeshlet {
    pub mesh_index: u32,
    pub indices_offset: u32,
    pub indices_count: u32,
    pub lod_level: u32,
    pub aabb_min: [f32; 3],
    pub parent_error: f32,
    pub aabb_max: [f32; 3],
    pub group_error: f32,
    pub bounding_sphere: [f32; 4],
    pub parent_bounding_sphere: [f32; 4],
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
pub struct GPUVertexIndices(pub u32);
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct GPUVertexPosition(pub u32);
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct GPUVertexAttributes(pub u32);

impl GPUVertexPosition {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder
    }
}
impl GPUVertexAttributes {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::vertex();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder
    }
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct GPUTransform {
    pub orientation: [f32; 4],
    pub position_scale_x: [f32; 4],
    pub bb_min_scale_y: [f32; 4],
    pub bb_max_scale_z: [f32; 4],
}

impl Default for GPUTransform {
    fn default() -> Self {
        Self {
            orientation: [0.; 4],
            position_scale_x: [0.; 4],
            bb_min_scale_y: [0.; 4],
            bb_max_scale_z: [0.; 4],
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GPUInstance {
    pub transform_id: u32,
    pub mesh_id: u32,
    pub meshlet_id: u32,
    pub command_id: i32,
}

impl GPUInstance {
    pub fn descriptor<'a>(starting_location: u32) -> VertexBufferLayoutBuilder<'a> {
        let mut layout_builder = VertexBufferLayoutBuilder::instance();
        layout_builder.starting_location(starting_location);
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<u32>(VertexFormat::Uint32.into());
        layout_builder.add_attribute::<i32>(VertexFormat::Sint32.into());
        layout_builder
    }
}

impl AsBinding for GPUInstance {
    fn count(&self) -> usize {
        1
    }

    fn size(&self) -> u64 {
        std::mem::size_of::<Self>() as u64
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut BufferRef) {
        buffer.add_to_gpu_buffer(
            render_context,
            &[
                self.transform_id,
                self.mesh_id,
                self.meshlet_id,
                self.command_id as _,
            ],
        );
    }
}
