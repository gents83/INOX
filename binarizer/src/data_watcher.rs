use std::path::{Path, PathBuf};

use nrg_platform::{FileEvent, FileWatcher};
use nrg_resources::get_absolute_path_from;

pub trait ExtensionHandler {
    fn on_changed(&mut self, path: &Path);
}

pub struct DataWatcher {
    filewatcher: FileWatcher,
    handlers: Vec<Box<dyn ExtensionHandler>>,
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
}

impl DataWatcher {
    pub fn new(data_raw_folder: &str, data_folder: &str) -> Self {
        let mut data_raw_folder = PathBuf::from(data_raw_folder);
        let mut data_folder = PathBuf::from(data_folder);
        data_raw_folder = data_raw_folder.canonicalize().unwrap();
        data_folder = data_folder.canonicalize().unwrap();
        debug_assert!(
            data_raw_folder.exists() && data_raw_folder.is_dir() && data_raw_folder.is_absolute()
        );
        debug_assert!(data_folder.exists() && data_folder.is_dir() && data_folder.is_absolute());
        Self {
            filewatcher: FileWatcher::new(data_raw_folder.clone()),
            handlers: Vec::new(),
            data_raw_folder,
            data_folder,
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
            self.binarize_file(path.as_path());
        }
    }

    pub fn binarize_all(&mut self) {
        let path = self.data_raw_folder.clone();
        self.binarize_folder(path.as_path());
    }

    fn binarize_file(&mut self, path: &Path) {
        let absolute_path = get_absolute_path_from(self.data_raw_folder.as_path(), path);
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
