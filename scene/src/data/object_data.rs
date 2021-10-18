use std::path::PathBuf;

use nrg_math::{MatBase, Matrix4};
use nrg_serialize::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct ObjectData {
    pub transform: Matrix4,
    pub mesh: PathBuf,
    pub children: Vec<PathBuf>,
}

impl Default for ObjectData {
    fn default() -> Self {
        Self {
            transform: Matrix4::default_identity(),
            mesh: PathBuf::new(),
            children: Vec::new(),
        }
    }
}
