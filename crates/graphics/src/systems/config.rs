use std::path::PathBuf;

use inox_resources::ConfigBase;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::RenderPassData;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "inox_serialize")]
pub struct Config {
    pub render_passes: Vec<RenderPassData>,
    pub default_pipeline: PathBuf,
    pub wireframe_pipeline: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            render_passes: Vec::new(),
            default_pipeline: PathBuf::new(),
            wireframe_pipeline: PathBuf::new(),
        }
    }
}

impl SerializeFile for Config {
    fn extension() -> &'static str {
        "cfg"
    }
}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "render.cfg"
    }
}
