use std::path::PathBuf;

use inox_resources::ConfigBase;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "inox_serialize")]
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
