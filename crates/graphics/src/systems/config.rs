use inox_resources::ConfigBase;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::RenderPassData;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "inox_serialize")]
pub struct Config {
    pub render_passes: Vec<RenderPassData>,
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
