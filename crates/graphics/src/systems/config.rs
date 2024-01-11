use std::path::PathBuf;

use inox_resources::ConfigBase;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "inox_serialize")]
pub struct Config {
    pub wireframe_pipeline: PathBuf,
    pub lut_pbr_charlie: PathBuf,
    pub lut_pbr_ggx: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wireframe_pipeline: PathBuf::new(),
            lut_pbr_charlie: PathBuf::new(),
            lut_pbr_ggx: PathBuf::new(),
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
