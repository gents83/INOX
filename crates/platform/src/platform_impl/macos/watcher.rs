use std::path::{Path, PathBuf};

pub struct FileWatcherImpl;

impl FileWatcherImpl {
    pub fn new<F: Fn(crate::watcher::FileEvent) + Send + 'static>(_f: F) -> Result<Self, String> {
        Ok(Self)
    }
    pub fn unwatch(&mut self, _path: &Path) {}
    pub fn stop(&mut self) {}
    pub fn watch(&mut self, _path: &Path) {}
}
