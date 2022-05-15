use std::path::PathBuf;

use inox_serialize::{Deserialize, Serialize, SerializeFile};

#[repr(C)]
#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct ComputePassData {
    pub name: String,
    pub pipelines: Vec<PathBuf>,
}

impl SerializeFile for ComputePassData {
    fn extension() -> &'static str {
        "compute_pass"
    }
}

unsafe impl Send for ComputePassData {}
unsafe impl Sync for ComputePassData {}
