use inox_serialize::{Deserialize, Serialize, SerializeFile};

pub const SHADER_EXTENSION: &str = "shader";

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
