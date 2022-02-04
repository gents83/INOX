use inox_serialize::{Deserialize, Serialize, SerializeFile};
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct SceneData {
    pub objects: Vec<PathBuf>,
    pub cameras: Vec<PathBuf>,
    pub lights: Vec<PathBuf>,
}

impl SerializeFile for SceneData {
    fn extension() -> &'static str {
        "scene"
    }
}
