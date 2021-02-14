use std::path::PathBuf;

use nrg_core::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct VkData {
    pub debug_validation_layers: bool,
}

impl Default for VkData {
    fn default() -> Self {
        Self {
            debug_validation_layers: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct PipelineData {
    pub name: String,
    pub fragment_shader: PathBuf,
    pub vertex_shader: PathBuf,
}

impl Default for PipelineData {
    fn default() -> Self {
        Self {
            name: String::from("Default"),
            fragment_shader: PathBuf::new(),
            vertex_shader: PathBuf::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct Config {
    pub vk_data: VkData,
    pub pipelines: Vec<PipelineData>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vk_data: VkData::default(),
            pipelines: Vec::new(),
        }
    }
}

impl Data for Config {}
impl ConfigBase for Config {}

impl Config {
    pub fn get_pipeline_data(&self, name: String) -> Option<&PipelineData> {
        self.pipelines.iter().find(|&el| el.name == name)
    }
}
