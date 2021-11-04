use std::path::PathBuf;

use nrg_resources::{ConfigBase, Data};
use nrg_serialize::{Deserialize, Serialize};

use crate::RenderPassData;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct Config {
    pub render_passes: Vec<RenderPassData>,
    pub pipelines: Vec<PathBuf>,
}

impl Data for Config {}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "render.cfg"
    }
}
