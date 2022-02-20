use std::path::PathBuf;

use crate::NormalizedPath;

impl NormalizedPath for PathBuf {
    fn normalize(&self) -> PathBuf {
        self.to_path_buf()
    }
}
