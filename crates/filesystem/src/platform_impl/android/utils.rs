use crate::NormalizedPath;
use std::path::{Path, PathBuf};

impl NormalizedPath for PathBuf {
    fn normalize(&self) -> PathBuf {
        self.to_path_buf()
    }
}

#[inline]
pub fn convert_from_local_path(parent_folder: &Path, relative_path: &Path) -> PathBuf {
    let string = relative_path.to_str().unwrap().to_string();
    let string = string.replace('\\', "/");
    let relative_path = PathBuf::from(string.clone());
    if string.contains(parent_folder.to_str().unwrap()) {
        relative_path
    } else {
        parent_folder.join(relative_path)
    }
}
