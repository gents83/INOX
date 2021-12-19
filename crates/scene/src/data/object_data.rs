use std::path::PathBuf;

use sabi_math::{MatBase, Matrix4};
use sabi_serialize::*;

#[derive(Serializable, Debug, PartialEq, Clone)]
pub struct ObjectData {
    pub transform: [[f32; 4]; 4],
    pub components: Vec<PathBuf>,
    pub children: Vec<PathBuf>,
}

impl SerializeFile for ObjectData {
    fn extension() -> &'static str {
        "object"
    }
}

impl Default for ObjectData {
    fn default() -> Self {
        Self {
            transform: Matrix4::default_identity().into(),
            components: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl ObjectData {
    pub fn transform(&self) -> Matrix4 {
        self.transform.into()
    }
}
