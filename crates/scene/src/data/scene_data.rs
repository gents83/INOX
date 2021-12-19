use sabi_serialize::*;
use std::path::PathBuf;

#[derive(Default, Serializable, Debug, PartialEq, Clone)]
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
