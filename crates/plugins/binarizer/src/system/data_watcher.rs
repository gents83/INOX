use std::path::{Path, PathBuf};

use inox_filesystem::convert_from_local_path;
use inox_platform::FileWatcher;

pub trait ExtensionHandler {
    fn on_changed(&mut self, path: &Path);
}

pub struct DataWatcher {
    filewatcher: FileWatcher,
    handlers: Vec<Box<dyn ExtensionHandler>>,
    data_raw_folder: PathBuf,
}

unsafe impl Send for DataWatcher {}
unsafe impl Sync for DataWatcher {}

impl DataWatcher {
    pub fn new(data_raw_folder: PathBuf) -> Self {
        Self {
            filewatcher: FileWatcher::new(data_raw_folder.clone()),
            handlers: Vec::new(),
            data_raw_folder,
        }
    }
    pub fn add_handler<H>(&mut self, handler: H)
    where
        H: ExtensionHandler + 'static,
    {
        self.handlers.push(Box::new(handler));
    }

    pub fn update(&mut self) {
        while let Ok(path) = self.filewatcher.read_events().try_recv() {
            let path = std::path::PathBuf::from(path);
            if path.is_file() {
                self.binarize_file(path.as_path());
            }
        }
    }

    pub fn binarize_all(&mut self) {
        let path = self.data_raw_folder.clone();
        self.binarize_folder(path.as_path());
    }

    fn binarize_file(&mut self, path: &Path) {
        let absolute_path = convert_from_local_path(self.data_raw_folder.as_path(), path);
        for handler in self.handlers.iter_mut() {
            handler.on_changed(absolute_path.as_path());
        }
    }

    fn binarize_folder(&mut self, path: &Path) {
        if let Ok(dir) = std::fs::read_dir(path) {
            dir.for_each(|entry| {
                if let Ok(dir_entry) = entry {
                    let path = dir_entry.path();
                    if !path.is_dir() {
                        self.binarize_file(path.as_path());
                    } else {
                        self.binarize_folder(path.as_path());
                    }
                }
            });
        }
    }
}

impl Drop for DataWatcher {
    fn drop(&mut self) {
        self.filewatcher.stop();
    }
}
