use nrg_resources::{ConfigBase, Data};
use nrg_serialize::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct Config {
    pub title: String,
    pub pos_x: u32,
    pub pos_y: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: String::new(),
            pos_x: 0,
            pos_y: 0,
            width: 1280,
            height: 720,
        }
    }
}

impl Data for Config {}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "app.cfg"
    }
}
