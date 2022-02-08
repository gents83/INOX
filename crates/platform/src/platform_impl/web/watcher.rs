use crate::watcher::*;
use std::path::Path;

pub struct FileWatcherImpl {}

impl FileWatcherImpl {
    pub fn new<F: EventFn>(_event_func: F) -> Result<Self> {
        eprintln!("Trying to create a FileWatcher on a non-supported platform");
        Ok(Self {})
    }

    #[inline]
    pub fn watch(&mut self, _path: &Path) {}

    #[inline]
    pub fn unwatch(&mut self, _path: &Path) {}
}
