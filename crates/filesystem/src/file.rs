use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

pub struct File {
    pub path: PathBuf,
    pub bytes: Arc<RwLock<Vec<u8>>>,
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
}
