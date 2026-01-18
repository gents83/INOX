use crate::watcher::FileEvent;
use std::path::Path;

pub struct FileWatcherImpl {}

impl FileWatcherImpl {
    pub fn new<F>(_event_fn: F) -> Result<Self, String>
    where
        F: Fn(FileEvent) + Send + 'static,
    {
        Ok(FileWatcherImpl {})
    }
    pub fn watch(&mut self, _path: &Path) {}
    pub fn unwatch(&mut self, _path: &Path) {}
}
