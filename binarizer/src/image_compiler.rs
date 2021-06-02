use std::path::{Path, PathBuf};

use crate::ExtensionHandler;
use nrg_messenger::{Message, MessengerRw};
use nrg_resources::{ResourceEvent, DATA_FOLDER, DATA_RAW_FOLDER};

const IMAGE_PNG_EXTENSION: &str = "png";
const IMAGE_JPG_EXTENSION: &str = "jpg";
const IMAGE_JPEG_EXTENSION: &str = "jpeg";

pub struct ImageCompiler {
    global_messenger: MessengerRw,
}

impl ImageCompiler {
    pub fn new(global_messenger: MessengerRw) -> Self {
        Self { global_messenger }
    }

    fn copy_into_data_folder(&self, path: &Path) -> bool {
        let mut from_source_to_compiled = path.to_str().unwrap().to_string();
        from_source_to_compiled = from_source_to_compiled.replace(
            PathBuf::from(DATA_RAW_FOLDER)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            PathBuf::from(DATA_FOLDER)
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        let new_path = PathBuf::from(from_source_to_compiled);
        if !new_path.exists() {
            let result = std::fs::create_dir_all(new_path.parent().unwrap());
            debug_assert!(result.is_ok());
        }
        let result = std::fs::copy(path, new_path.as_path());
        if result.is_ok() {
            let dispatcher = self.global_messenger.read().unwrap().get_dispatcher();
            dispatcher
                .write()
                .unwrap()
                .send(ResourceEvent::Reload(new_path).as_boxed())
                .ok();
            return true;
        }
        false
    }
}

impl ExtensionHandler for ImageCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            let extension = ext.to_str().unwrap().to_string();
            if extension.as_str() == IMAGE_PNG_EXTENSION
                || extension.as_str() == IMAGE_JPG_EXTENSION
                || extension.as_str() == IMAGE_JPEG_EXTENSION
            {
                self.copy_into_data_folder(path);
            }
        }
    }
}
