use std::path::PathBuf;

use nrg_core::*;
use nrg_graphics::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct Config {
    pub fonts: Vec<PathBuf>,
    pub pipelines: Vec<PipelineData>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fonts: Vec::new(),
            pipelines: Vec::new(),
        }
    }
}

impl Data for Config {}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "gui.cfg"
    }
}
