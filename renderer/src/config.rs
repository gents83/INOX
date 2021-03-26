use nrg_core::*;
use nrg_graphics::*;
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
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "graphics.cfg"
    }
}
