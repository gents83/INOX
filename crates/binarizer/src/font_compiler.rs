use std::path::Path;

use crate::{copy_into_data_folder, ExtensionHandler};
use inox_messenger::MessageHubRc;
use inox_profiler::debug_log;

const FONT_EXTENSION: &str = "ttf";

pub struct FontCompiler {
    message_hub: MessageHubRc,
}

impl FontCompiler {
    pub fn new(message_hub: MessageHubRc) -> Self {
        Self { message_hub }
    }
}

impl ExtensionHandler for FontCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            if ext.to_str().unwrap().to_string().as_str() == FONT_EXTENSION
                && copy_into_data_folder(&self.message_hub, path)
            {
                debug_log!("Serializing {:?}", path);
            }
        }
    }
}
