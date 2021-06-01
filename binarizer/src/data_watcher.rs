use std::path::{Path, PathBuf};

use nrg_platform::{FileEvent, FileWatcher};
use nrg_resources::get_absolute_data_path;

pub trait ExtensionHandler {
    fn on_changed(&mut self, path: &Path);
}

pub struct DataWatcher {
    filewatcher: FileWatcher,
    handlers: Vec<Box<dyn ExtensionHandler>>,
}

impl DataWatcher {
    pub fn new(folder: &str) -> Self {
        Self {
            filewatcher: FileWatcher::new(PathBuf::from(folder)),
            handlers: Vec::new(),
        }
    }

    pub fn add_handler<H>(&mut self, handler: H)
    where
        H: ExtensionHandler + 'static,
    {
        self.handlers.push(Box::new(handler));
    }

    pub fn update(&mut self) {
        while let Ok(FileEvent::Modified(path)) = self.filewatcher.read_events().try_recv() {
            for handler in self.handlers.iter_mut() {
                handler.on_changed(get_absolute_data_path(path.as_path()).as_path());
            }
        }
    }
}

impl Drop for DataWatcher {
    fn drop(&mut self) {
        self.filewatcher.stop();
    }
}
