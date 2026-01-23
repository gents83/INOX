use std::path::{Path, PathBuf};

pub struct FileWatcherImpl;

impl FileWatcherImpl {
    pub fn new(_path: PathBuf, _f: impl Fn(&str) + Send + 'static) -> Result<Self, String> {
        Ok(Self)
    }
    pub fn unwatch(&mut self, _path: &Path) {}
    pub fn stop(&mut self) {}
    pub fn watch(&mut self, _path: &Path) {}
}
