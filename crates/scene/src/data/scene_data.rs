use nrg_serialize::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct SceneData {
    pub objects: Vec<PathBuf>,
    pub cameras: Vec<PathBuf>,
    pub lights: Vec<PathBuf>,
}
