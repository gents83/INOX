use crate::watcher::{EventFn, Result};
use std::path::Path;

pub struct FileWatcherImpl;

impl FileWatcherImpl {
    pub fn new<F: EventFn>(_event_func: F) -> Result<Self> {
        Ok(Self)
    }

    #[inline]
    pub fn watch(&mut self, _path: &Path) {
    }

    #[inline]
    pub fn unwatch(&mut self, _path: &Path) {
    }
}

impl Drop for FileWatcherImpl {
    fn drop(&mut self) {
    }
}
