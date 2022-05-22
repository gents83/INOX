use std::path::PathBuf;

use inox_filesystem::convert_from_local_path;

use inox_resources::Data;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct ComputePipelineData {
    pub shader: PathBuf,
}

impl SerializeFile for ComputePipelineData {
    fn extension() -> &'static str {
        "compute_pipeline"
    }
}

impl Default for ComputePipelineData {
    fn default() -> Self {
        Self {
            shader: PathBuf::new(),
        }
    }
}

impl ComputePipelineData {
    pub fn is_valid(&self) -> bool {
        !self.shader.to_str().unwrap_or_default().is_empty()
    }
    pub fn canonicalize_paths(mut self) -> Self {
        let data_path = Data::platform_data_folder();
        if !self.shader.to_str().unwrap().is_empty() {
            self.shader = convert_from_local_path(data_path.as_path(), self.shader.as_path());
        }
        self
    }
    pub fn has_same_shaders(&self, other: &ComputePipelineData) -> bool {
        self.shader == other.shader
    }
}
