use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

pub struct File {
    #[allow(dead_code)]
    pub(crate) path: PathBuf,
    pub(crate) bytes: Arc<RwLock<Vec<u8>>>,
}

impl File {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            bytes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn apply<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(&mut Vec<u8>) + 'static,
    {
        {
            let mut bytes = self.bytes.write().unwrap();
            let bytes = bytes.as_mut();
            f(bytes);
        }
        self
    }

    pub fn is_loaded(&self) -> bool {
        !self.bytes.read().unwrap().is_empty()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.read().unwrap().clone()
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn load<F>(&self, f: F)
    where
        F: FnMut(&[u8]),
    {
        let mut f = f;
        if self.path.exists() {
            if let Ok(bytes) = std::fs::read(&self.path) {
                *self.bytes.write().unwrap() = bytes;
            }
        }
        let bytes = self.bytes.read().unwrap();
        f(&bytes);
    }

    pub fn save<F>(&self, f: F)
    where
        F: FnMut(&mut Vec<u8>),
    {
        let mut f = f;
        {
            let mut bytes = self.bytes.write().unwrap();
            f(&mut bytes);
        }
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        let bytes = self.bytes.read().unwrap();
        let _ = std::fs::write(&self.path, &*bytes);
    }
}
