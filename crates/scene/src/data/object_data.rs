use std::path::PathBuf;

use nrg_math::{MatBase, Matrix4};
use nrg_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct ObjectData {
    pub transform: Matrix4,
    pub components: Vec<PathBuf>,
    pub children: Vec<PathBuf>,
}

impl SerializeFile for ObjectData {
    fn extension() -> &'static str {
        "object_data"
    }
}

impl Default for ObjectData {
    fn default() -> Self {
        Self {
            transform: Matrix4::default_identity(),
            components: Vec::new(),
            children: Vec::new(),
        }
    }
}
