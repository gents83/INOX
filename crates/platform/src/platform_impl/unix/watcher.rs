use std::path::Path;
use crate::watcher::{EventFn, Result};

pub struct FileWatcherImpl;

impl FileWatcherImpl {
    pub fn new<F: EventFn>(_event_fn: F) -> Result<Self> {
        Ok(FileWatcherImpl)
    }

    pub fn watch(&mut self, _path: &Path) {
    }

    pub fn unwatch(&mut self, _path: &Path) {
    }
}
