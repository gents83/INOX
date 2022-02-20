use std::path::PathBuf;

use crate::NormalizedPath;

impl NormalizedPath for PathBuf {
    fn normalize(&self) -> PathBuf {
        self.canonicalize().unwrap_or_else(|_| {
            let path = self.to_str().unwrap().to_string();
            let win_prefix = "\\\\?\\".to_string();
            let string = if path.starts_with(&win_prefix) {
                path
            } else {
                win_prefix + &path
            };
            PathBuf::from(string)
        })
    }
}
