use std::path::{Path, PathBuf};

use crate::{copy_into_data_folder, ExtensionHandler};
use inox_log::debug_log;
use inox_messenger::MessageHubRc;

const FONT_EXTENSION: &str = "ttf";

pub struct FontCompiler {
    message_hub: MessageHubRc,
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
}

impl FontCompiler {
    pub fn new(message_hub: MessageHubRc, data_raw_folder: &Path, data_folder: &Path) -> Self {
        Self {
            message_hub,
            data_raw_folder: data_raw_folder.to_path_buf(),
            data_folder: data_folder.to_path_buf(),
        }
    }
}

impl ExtensionHandler for FontCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            if ext.to_str().unwrap().to_string().as_str() == FONT_EXTENSION
                && copy_into_data_folder(
                    &self.message_hub,
                    path,
                    self.data_raw_folder.as_path(),
                    self.data_folder.as_path(),
                )
            {
                debug_log!("Serializing {:?}", path);
            }
        }
    }
}
