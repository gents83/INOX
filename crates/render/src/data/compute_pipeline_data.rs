use std::path::PathBuf;

use inox_filesystem::convert_from_local_path;

use inox_resources::Data;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
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
    pub fn canonicalize_paths(&self) -> Self {
        let mut data = self.clone();
        let data_path = Data::platform_data_folder();
        if !data.shader.to_str().unwrap().is_empty() {
            data.shader = convert_from_local_path(data_path.as_path(), data.shader.as_path());
        }
        data
    }
    pub fn has_same_shaders(&self, other: &ComputePipelineData) -> bool {
        self.shader == other.shader
    }
}
