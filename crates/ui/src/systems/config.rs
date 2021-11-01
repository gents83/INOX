use std::path::PathBuf;

use nrg_resources::{ConfigBase, Data};
use nrg_serialize::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct Config {
    pub ui_scale: f32,
    pub ui_pipeline: PathBuf,
}

impl Data for Config {}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "ui.cfg"
    }
}
