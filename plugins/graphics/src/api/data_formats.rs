use std::path::PathBuf;

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
pub struct PipelineData {
    pub name: String,
    pub fragment_shader: PathBuf,
    pub vertex_shader: PathBuf,
}
unsafe impl Send for PipelineData {}
unsafe impl Sync for PipelineData {}

impl Default for PipelineData {
    fn default() -> Self {
        Self {
            name: String::from("Default"),
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
