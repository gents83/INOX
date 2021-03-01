use std::path::PathBuf;

use crate::common::utils::*;

use nrg_math::*;
use nrg_serialize::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct UniformData {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct VertexData {
    pub pos: Vector3f,
    pub color: Vector3f,
    pub tex_coord: Vector2f,
    pub normal: Vector3f,
}

impl Default for VertexData {
    fn default() -> VertexData {
        VertexData {
            pos: [0.0, 0.0, 0.0].into(),
            color: [1.0, 1.0, 1.0].into(),
            tex_coord: [0.0, 0.0].into(),
            normal: [0.0, 0.0, 1.0].into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct RenderPassData {
    pub clear: bool,
    pub index: i32,
}
unsafe impl Send for RenderPassData {}
unsafe impl Sync for RenderPassData {}

impl Default for RenderPassData {
    fn default() -> Self {
        Self {
            clear: true,
            index: -1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct PipelineData {
    pub name: String,
    pub data: RenderPassData,
    pub fragment_shader: PathBuf,
    pub vertex_shader: PathBuf,
}
unsafe impl Send for PipelineData {}
unsafe impl Sync for PipelineData {}

impl Default for PipelineData {
    fn default() -> Self {
        Self {
            name: String::from("Default"),
            data: RenderPassData::default(),
            fragment_shader: PathBuf::new(),
            vertex_shader: PathBuf::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct MaterialData {
    pub pipeline_id: String,
    pub textures: Vec<PathBuf>,
}
unsafe impl Send for MaterialData {}
unsafe impl Sync for MaterialData {}

impl Default for MaterialData {
    fn default() -> Self {
        Self {
            pipeline_id: String::from("Default"),
            textures: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct MeshData {
    pub center: Vector3f,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            center: Vector3f::default(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl MeshData {
    pub fn clear(&mut self) -> &mut Self {
        self.vertices.clear();
        self.indices.clear();
        self
    }

    pub fn compute_center(&mut self) -> &mut Self {
        let mut min = Vector3f {
            x: Float::max_value(),
            y: Float::max_value(),
            z: Float::max_value(),
        };
        let mut max = Vector3f {
            x: Float::min_value(),
            y: Float::min_value(),
            z: Float::min_value(),
        };
        for v in self.vertices.iter() {
            min.x = min.x.min(v.pos.x);
            min.y = min.y.min(v.pos.y);
            min.z = min.z.min(v.pos.z);
            max.x = max.x.max(v.pos.x);
            max.y = max.y.max(v.pos.y);
            max.z = max.z.max(v.pos.z);
        }
        self.center.x = min.x + (max.x - min.x) * 0.5;
        self.center.y = min.y + (max.y - min.y) * 0.5;
        self.center.z = min.z + (max.z - min.z) * 0.5;
        self
    }

    pub fn set_vertices(&mut self, vertex_data: &[VertexData]) -> &mut Self {
        self.vertices.clear();
        self.vertices.extend_from_slice(vertex_data);
        self.compute_center();
        self
    }

    pub fn set_indices(&mut self, indices_data: &[u32]) -> &mut Self {
        self.indices.clear();
        self.indices.extend_from_slice(indices_data);
        self
    }

    pub fn set_vertex_color(&mut self, color: Vector3f) -> &mut Self {
        for v in self.vertices.iter_mut() {
            v.color = color;
        }
        self
    }

    pub fn translate(&mut self, movement: Vector3f) -> &mut Self {
        self.vertices.iter_mut().for_each(|v| {
            v.pos.x += movement.x;
            v.pos.y += movement.y;
            v.pos.z += movement.z;
        });
        self.compute_center();
        self
    }

    pub fn scale(&mut self, scale: Vector3f) -> &mut Self {
        self.vertices.iter_mut().for_each(|v| {
            v.pos.x *= scale.x;
            v.pos.y *= scale.y;
            v.pos.z *= scale.z;
        });
        self.compute_center();
        self
    }

    pub fn add_quad_default(&mut self, rect: Vector4f, z: f32) -> &mut Self {
        let tex_coords = [0.0, 0.0, 1.0, 1.0].into();
        let (vertices, indices) = create_quad(rect, z, tex_coords, None);

        self.vertices.append(&mut vertices.to_vec());
        self.indices.append(&mut indices.to_vec());
        self.compute_center();
        self
    }

    pub fn add_quad(
        &mut self,
        rect: Vector4f,
        z: f32,
        tex_coords: Vector4f,
        index_start: Option<usize>,
    ) -> &mut Self {
        let (vertices, indices) = create_quad(rect, z, tex_coords, index_start);

        self.vertices.append(&mut vertices.to_vec());
        self.indices.append(&mut indices.to_vec());
        self.compute_center();
        self
    }
}
