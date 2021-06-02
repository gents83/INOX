#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ShaderType {
    Invalid,
    Vertex,
    Fragment,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
}

pub const SHADER_EXTENSION: &str = "spv";
