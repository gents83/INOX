use nrg_graphics::PipelineData;
use nrg_resources::{ConfigBase, Data};
use nrg_serialize::*;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct Config {
    pub pos_x: u32,
    pub pos_y: u32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub fonts: Vec<PathBuf>,
    pub pipelines: Vec<PipelineData>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pos_x: 0,
            pos_y: 0,
            width: 1280,
            height: 720,
            scale_factor: 1.,
            fonts: Vec::new(),
            pipelines: Vec::new(),
        }
    }
}

impl Data for Config {}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "launcher.cfg"
    }
}
