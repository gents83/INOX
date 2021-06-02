use std::{
    fs::{copy, create_dir_all},
    path::{Path, PathBuf},
};

use crate::{need_to_binarize, send_reloaded_event, ExtensionHandler};
use nrg_messenger::MessengerRw;
use nrg_resources::{DATA_FOLDER, DATA_RAW_FOLDER};

const FONT_EXTENSION: &str = "ttf";

pub struct FontCompiler {
    global_messenger: MessengerRw,
}

impl FontCompiler {
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
            let result = create_dir_all(new_path.parent().unwrap());
            debug_assert!(result.is_ok());
        }
        if need_to_binarize(path, new_path.as_path()) {
            let result = copy(path, new_path.as_path());
            if result.is_ok() {
                send_reloaded_event(&self.global_messenger, new_path.as_path());
                return true;
            }
        }
        false
    }
}

impl ExtensionHandler for FontCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            if ext.to_str().unwrap().to_string().as_str() == FONT_EXTENSION {
                self.copy_into_data_folder(path);
            }
        }
    }
}
