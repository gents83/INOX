use std::path::{Path, PathBuf};

use nrg_math::{MatBase, Matrix4};
use nrg_resources::implement_file_data;
use nrg_serialize::*;

implement_file_data!(
    struct ObjectData {
        transform: Matrix4,
        mesh: PathBuf,
        material: PathBuf,
        children: Vec<PathBuf>,
    }
);

impl Default for ObjectData {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            transform: Matrix4::default_identity(),
            mesh: PathBuf::new(),
            material: PathBuf::new(),
            children: Vec::new(),
        }
    }
}
