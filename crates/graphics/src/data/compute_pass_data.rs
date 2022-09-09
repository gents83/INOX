use std::path::PathBuf;

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct ComputePassData {
    pub name: String,
    pub pipelines: Vec<PathBuf>,
}

unsafe impl Send for ComputePassData {}
unsafe impl Sync for ComputePassData {}
