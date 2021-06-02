use std::path::Path;

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

pub fn is_shader(path: &Path) -> bool {
    path.extension().unwrap() == SHADER_EXTENSION
}
