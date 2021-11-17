use std::path::PathBuf;

use sabi_resources::{ConfigBase, Data};
use sabi_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct Config {
    pub grid_material: PathBuf,
}

impl Data for Config {}
impl SerializeFile for Config {
    fn extension() -> &'static str {
        "cfg"
    }
}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "editor.cfg"
    }
}
