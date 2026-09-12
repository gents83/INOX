use crate::watcher::{EventFn, Result};
use std::path::Path;

pub struct FileWatcherImpl;

impl FileWatcherImpl {
    pub fn new<F: EventFn>(_event_fn: F) -> Result<Self> {
        Ok(FileWatcherImpl)
    }

    pub fn watch(&mut self, _path: &Path) {}

    pub fn unwatch(&mut self, _path: &Path) {}
}
