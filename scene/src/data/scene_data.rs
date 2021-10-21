use nrg_serialize::*;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct SceneData {
    pub objects: Vec<PathBuf>,
    pub cameras: Vec<PathBuf>,
    pub lights: Vec<PathBuf>,
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            cameras: Vec::new(),
            lights: Vec::new(),
        }
    }
}
