use std::path::PathBuf;

use nrg_math::{MatBase, Matrix4};
use nrg_serialize::*;

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct ObjectData {
    pub transform: Matrix4,
    pub material: PathBuf,
    pub children: Vec<PathBuf>,
}
unsafe impl Send for ObjectData {}
unsafe impl Sync for ObjectData {}

impl Default for ObjectData {
    fn default() -> Self {
        Self {
            transform: Matrix4::default_identity(),
            material: PathBuf::new(),
            children: Vec::new(),
        }
    }
}
