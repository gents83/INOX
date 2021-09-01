use std::path::Path;

use crate::{copy_into_data_folder, ExtensionHandler};
use nrg_messenger::MessengerRw;

const FONT_EXTENSION: &str = "ttf";

pub struct FontCompiler {
    global_messenger: MessengerRw,
}

impl FontCompiler {
    pub fn new(global_messenger: MessengerRw) -> Self {
        Self { global_messenger }
    }
}

impl ExtensionHandler for FontCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            if ext.to_str().unwrap().to_string().as_str() == FONT_EXTENSION
                && copy_into_data_folder(&self.global_messenger, path)
            {
                println!("Serializing {:?}", path);
            }
        }
    }
}
