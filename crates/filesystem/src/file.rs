use std::path::{Path, PathBuf};

use crate::platform_impl::platform::file;

#[derive(Clone)]
pub struct File {
    #[allow(dead_code)]
    pub(crate) path: PathBuf,
}

impl File {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn load(&self, f: impl FnOnce(&mut [u8])) {
        if let Ok(mut bytes) = std::fs::read(&self.path) {
            f(&mut bytes);
        }
    }

    pub fn save(&self, f: impl FnOnce(&mut Vec<u8>)) {
        let mut bytes = Vec::new();
        f(&mut bytes);
        let _ = std::fs::write(&self.path, bytes);
    }

    pub fn save_ip(path: &Path, ip: &str) {
        if path.exists() {
            let _res = std::fs::remove_file(path);
        }
        let _res = std::fs::write(path, ip);
    }

    pub fn get_exe_path() -> PathBuf {
        file::get_exe_path()
    }
}
