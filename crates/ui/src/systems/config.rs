use std::path::PathBuf;

use sabi_resources::ConfigBase;
use sabi_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct Config {
    pub ui_scale: f32,
    pub ui_pipeline: PathBuf,
}

impl SerializeFile for Config {
    fn extension() -> &'static str {
        "cfg"
    }
}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "ui.cfg"
    }
}
