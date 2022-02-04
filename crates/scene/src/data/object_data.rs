use std::path::PathBuf;

use inox_math::{MatBase, Matrix4};
use inox_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct ObjectData {
    pub transform: Matrix4,
    pub components: Vec<PathBuf>,
    pub children: Vec<PathBuf>,
}

impl SerializeFile for ObjectData {
    fn extension() -> &'static str {
        "object"
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
