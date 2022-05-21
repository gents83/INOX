use inox_serialize::{Deserialize, Serialize, SerializeFile};

pub const SHADER_EXTENSION: &str = "shader";

#[derive(Clone, Copy)]
pub enum ShaderStage {
    None,
    Vertex,
    Fragment,
    Compute,
    VertexAndFragment,
}

impl From<ShaderStage> for wgpu::ShaderStages {
    fn from(val: ShaderStage) -> Self {
        match val {
            ShaderStage::None => wgpu::ShaderStages::NONE,
            ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
            ShaderStage::Compute => wgpu::ShaderStages::COMPUTE,
            ShaderStage::VertexAndFragment => {
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT
            }
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct ShaderData {
    pub spirv_code: Vec<u32>,
    pub wgsl_code: String,
}

impl SerializeFile for ShaderData {
    fn extension() -> &'static str {
        SHADER_EXTENSION
    }
}
