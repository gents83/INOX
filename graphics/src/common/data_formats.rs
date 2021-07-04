use std::path::PathBuf;

use crate::common::utils::*;

use nrg_math::*;
use nrg_resources::{convert_from_local_path, DATA_FOLDER};
use nrg_serialize::*;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct InstanceCommand {
    pub mesh_index: usize,
    pub mesh_data_ref: MeshDataRef,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct InstanceData {
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
    pub draw_area: Vector4,
    pub diffuse_color: Vector4,
    pub diffuse_texture_index: i32,
    pub diffuse_layer_index: i32,
    pub outline_color: Vector4,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            position: Vector3::default_zero(),
            rotation: Vector3::default_zero(),
            scale: [1., 1., 1.].into(),
            draw_area: [0., 0., f32::MAX, f32::MAX].into(),
            diffuse_color: [1., 1., 1., 1.].into(),
            diffuse_texture_index: -1,
            diffuse_layer_index: -1,
            outline_color: [1., 1., 1., 0.].into(),
        }
    }
}
#[repr(C, align(16))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct UniformData {
    pub view: Matrix4,
    pub proj: Matrix4,
}
#[repr(C, align(16))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ConstantData {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Default for ConstantData {
    fn default() -> Self {
        Self {
            view: [[0.; 4]; 4],
            proj: [[0.; 4]; 4],
            screen_width: 0.,
            screen_height: 0.,
        }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "nrg_serialize")]
pub struct VertexData {
    pub pos: Vector3,
    pub color: Vector4,
    pub tex_coord: Vector2,
    pub normal: Vector3,
}

impl Default for VertexData {
    fn default() -> VertexData {
        VertexData {
            pos: [0.0, 0.0, 0.0].into(),
            color: [1.0, 1.0, 1.0, 1.0].into(),
            tex_coord: [0.0, 0.0].into(),
            normal: [0.0, 0.0, 1.0].into(),
        }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct RenderPassData {
    pub clear: bool,
    pub clear_depth: bool,
    pub name: String,
}
unsafe impl Send for RenderPassData {}
unsafe impl Sync for RenderPassData {}

impl Default for RenderPassData {
    fn default() -> Self {
        Self {
            clear: true,
            clear_depth: true,
            name: String::new(),
        }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct PipelineData {
    pub name: String,
    pub fragment_shader: PathBuf,
    pub vertex_shader: PathBuf,
    pub tcs_shader: PathBuf,
    pub tes_shader: PathBuf,
    pub geometry_shader: PathBuf,
}
unsafe impl Send for PipelineData {}
unsafe impl Sync for PipelineData {}

impl Default for PipelineData {
    fn default() -> Self {
        Self {
            name: String::from("3D"),
            fragment_shader: PathBuf::new(),
            vertex_shader: PathBuf::new(),
            tcs_shader: PathBuf::new(),
            tes_shader: PathBuf::new(),
            geometry_shader: PathBuf::new(),
        }
    }
}

impl PipelineData {
    pub fn canonicalize_paths(mut self) -> Self {
        let data_path = PathBuf::from(DATA_FOLDER);
        if !self.vertex_shader.to_str().unwrap().is_empty() {
            self.vertex_shader =
                convert_from_local_path(data_path.as_path(), self.vertex_shader.as_path());
        }
        if !self.fragment_shader.to_str().unwrap().is_empty() {
            self.fragment_shader =
                convert_from_local_path(data_path.as_path(), self.fragment_shader.as_path());
        }
        if !self.tcs_shader.to_str().unwrap().is_empty() {
            self.tcs_shader =
                convert_from_local_path(data_path.as_path(), self.tcs_shader.as_path());
        }
        if !self.tes_shader.to_str().unwrap().is_empty() {
            self.tes_shader =
                convert_from_local_path(data_path.as_path(), self.tes_shader.as_path());
        }
        if !self.geometry_shader.to_str().unwrap().is_empty() {
            self.geometry_shader =
                convert_from_local_path(data_path.as_path(), self.geometry_shader.as_path());
        }
        self
    }
    pub fn has_same_shaders(&self, other: &PipelineData) -> bool {
        self.vertex_shader == other.vertex_shader
            && self.fragment_shader == other.fragment_shader
            && self.tcs_shader == other.tcs_shader
            && self.tes_shader == other.tes_shader
            && self.geometry_shader == other.geometry_shader
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct MaterialData {
    pub pipeline_name: String,
    pub meshes: Vec<PathBuf>,
    pub textures: Vec<PathBuf>,
    pub diffuse_color: Vector4,
    pub outline_color: Vector4,
}
unsafe impl Send for MaterialData {}
unsafe impl Sync for MaterialData {}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            pipeline_name: String::from("3D"),
            meshes: Vec::new(),
            textures: Vec::new(),
            diffuse_color: [1., 1., 1., 1.].into(),
            outline_color: [1., 1., 1., 0.].into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshDataRef {
    pub first_vertex: u32,
    pub last_vertex: u32,
    pub first_index: u32,
    pub last_index: u32,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct MeshData {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub transform: Matrix4,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            transform: Matrix4::default_identity(),
        }
    }
}

impl MeshData {
    pub fn clear(&mut self) -> &mut Self {
        self.vertices.clear();
        self.indices.clear();
        self
    }

    pub fn compute_center(&mut self) -> Vector3 {
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in self.vertices.iter() {
            min.x = min.x.min(v.pos.x);
            min.y = min.y.min(v.pos.y);
            min.z = min.z.min(v.pos.z);
            max.x = max.x.max(v.pos.x);
            max.y = max.y.max(v.pos.y);
            max.z = max.z.max(v.pos.z);
        }
        [
            min.x + (max.x - min.x) * 0.5,
            min.y + (max.y - min.y) * 0.5,
            min.z + (max.z - min.z) * 0.5,
        ]
        .into()
    }

    pub fn set_vertex_color(&mut self, color: Vector4) -> &mut Self {
        for v in self.vertices.iter_mut() {
            v.color = color;
        }
        self
    }
    pub fn set_mesh_at_index(
        &mut self,
        vertices: &[VertexData],
        first_vertex: u32,
        indices: &[u32],
        first_index: u32,
    ) -> MeshDataRef {
        let start = first_vertex as usize;
        let end = start + vertices.len();
        self.vertices[start..end].copy_from_slice(vertices);

        let start = first_index as usize;
        let end = start + indices.len();
        self.indices[start..end].copy_from_slice(indices);

        MeshDataRef {
            first_vertex,
            last_vertex: first_vertex + vertices.len() as u32,
            first_index,
            last_index: first_index + indices.len() as u32,
        }
    }

    pub fn append_mesh(&mut self, vertices: &[VertexData], indices: &[u32]) -> MeshDataRef {
        let first_vertex = self.vertices.len() as u32;
        let first_index = self.indices.len() as u32;

        self.vertices.append(&mut vertices.to_vec());
        self.indices.append(&mut indices.to_vec());
        self.compute_center();

        let last_vertex = self.vertices.len() as u32 - 1;
        let last_index = self.indices.len() as u32 - 1;

        MeshDataRef {
            first_vertex,
            last_vertex,
            first_index,
            last_index,
        }
    }

    pub fn add_quad_default(&mut self, rect: Vector4, z: f32) -> MeshDataRef {
        let tex_coords = [0.0, 0.0, 1.0, 1.0].into();
        let (vertices, indices) = create_quad(rect, z, tex_coords, None);
        self.append_mesh(&vertices, &indices)
    }

    pub fn add_quad(
        &mut self,
        rect: Vector4,
        z: f32,
        tex_coords: Vector4,
        index_start: Option<usize>,
    ) -> MeshDataRef {
        let (vertices, indices) = create_quad(rect, z, tex_coords, index_start);
        self.append_mesh(&vertices, &indices)
    }

    pub fn clip_in_rect(&mut self, clip_rect: Vector4) -> &mut Self {
        for v in self.vertices.iter_mut() {
            v.pos.x = v.pos.x.max(clip_rect.x);
            v.pos.x = v.pos.x.min(clip_rect.z);
            v.pos.y = v.pos.y.max(clip_rect.y);
            v.pos.y = v.pos.y.min(clip_rect.w);
        }
        self.compute_center();
        self
    }

    pub fn is_inside(&self, pos_in_screen_space: Vector2) -> bool {
        let mut i = 0;
        let count = self.indices.len();
        while i < count {
            let v1 = self.vertices[self.indices[i] as usize].pos.xy();
            let v2 = self.vertices[self.indices[i + 1] as usize].pos.xy();
            let v3 = self.vertices[self.indices[i + 2] as usize].pos.xy();
            if is_point_in_triangle(v1, v2, v3, pos_in_screen_space.x, pos_in_screen_space.y) {
                return true;
            }
            i += 3;
        }
        false
    }
}
