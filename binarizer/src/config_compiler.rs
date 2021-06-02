use std::path::Path;

use crate::{copy_into_data_folder, ExtensionHandler};
use nrg_messenger::MessengerRw;

const CONFIG_EXTENSION: &str = "cfg";

pub struct ConfigCompiler {
    global_messenger: MessengerRw,
}

impl ConfigCompiler {
    pub fn new(global_messenger: MessengerRw) -> Self {
        Self { global_messenger }
    }
}

impl ExtensionHandler for ConfigCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            if ext.to_str().unwrap().to_string().as_str() == CONFIG_EXTENSION {
                copy_into_data_folder(&self.global_messenger, path);
            }
        }
    }
}
