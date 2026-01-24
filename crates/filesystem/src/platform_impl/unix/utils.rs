use std::path::{Path, PathBuf};

use crate::NormalizedPath;

#[inline]
pub fn convert_from_local_path(parent_folder: &Path, relative_path: &Path) -> PathBuf {
    let mut pathbuf = parent_folder.to_path_buf();
    let data_folder = pathbuf.normalize().to_str().unwrap().to_string();
    let string = relative_path.to_str().unwrap().to_string();
    if string.contains(parent_folder.to_str().unwrap()) {
        pathbuf = relative_path.normalize()
    } else if string.contains(data_folder.as_str()) {
        pathbuf = relative_path.to_path_buf()
    } else if let Ok(result_path) = pathbuf.join(relative_path).canonicalize() {
        pathbuf = result_path;
    } else {
        pathbuf = relative_path.normalize();
    }
    pathbuf
}
