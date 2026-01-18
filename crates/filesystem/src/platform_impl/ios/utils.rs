#![cfg(target_os = "ios")]

use std::path::{Path, PathBuf};

pub trait PathExtensions {
    fn normalize(&self) -> PathBuf;
}

impl PathExtensions for PathBuf {
    fn normalize(&self) -> PathBuf {
        self.as_path().normalize()
    }
}

impl PathExtensions for Path {
    fn normalize(&self) -> PathBuf {
        self.to_path_buf()
    }
}
