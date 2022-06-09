use inox_serialize::{Deserialize, Serialize};

use crate::{TextureType, INVALID_INDEX, MAX_TEXTURE_COORDS_SETS};

// Pipeline has a list of meshes to process
// Meshes can switch pipeline at runtime
// Material doesn't know pipeline anymore
// Material is now generic data for several purposes

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawInstance {
    pub mesh_index: u32,
    pub matrix_index: u32,
    pub draw_area_index: i32,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub base_index: u32,
    pub vertex_offset: i32,
    pub base_instance: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawMesh {
    pub meshlet_offset: u32,
    pub meshlet_count: u32,
    pub material_index: i32,
    pub matrix_index: i32,
    pub mesh_flags: u32,
}

impl Default for DrawMesh {
    fn default() -> Self {
        Self {
            meshlet_offset: 0,
            meshlet_count: 0,
            material_index: INVALID_INDEX,
            matrix_index: INVALID_INDEX,
            mesh_flags: 0,
        }
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawMeshlet {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub indices_offset: u32,
    pub indices_count: u32,
    pub center_radius: [f32; 4],
    pub cone_axis_cutoff: [f32; 4],
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct DrawVertex {
    pub position_offset: u32,
    pub color_offset: i32,
    pub normal_offset: i32,
    pub tangent_offset: i32,
    pub uv_offset: [i32; MAX_TEXTURE_COORDS_SETS],
}

impl Default for DrawVertex {
    fn default() -> Self {
        Self {
            position_offset: 0,
            color_offset: INVALID_INDEX,
            normal_offset: INVALID_INDEX,
            tangent_offset: INVALID_INDEX,
            uv_offset: [INVALID_INDEX; MAX_TEXTURE_COORDS_SETS],
        }
    }
}
