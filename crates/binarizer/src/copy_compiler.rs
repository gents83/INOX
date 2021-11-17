use std::path::Path;

use crate::{copy_into_data_folder, ExtensionHandler};
use sabi_graphics::{Light, Material, Mesh, Pipeline};
use sabi_messenger::MessengerRw;
use sabi_profiler::debug_log;
use sabi_resources::SerializableResource;
use sabi_scene::{Camera, Object, Scene};

const CONFIG_EXTENSION: &str = "cfg";

pub struct CopyCompiler {
    global_messenger: MessengerRw,
}

impl CopyCompiler {
    pub fn new(global_messenger: MessengerRw) -> Self {
        Self { global_messenger }
    }
}

impl ExtensionHandler for CopyCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap().to_string();
            if (ext.as_str() == CONFIG_EXTENSION
                || ext.as_str() == Material::extension()
                || ext.as_str() == Mesh::extension()
                || ext.as_str() == Pipeline::extension()
                || ext.as_str() == Scene::extension()
                || ext.as_str() == Object::extension()
                || ext.as_str() == Camera::extension()
                || ext.as_str() == Light::extension())
                && copy_into_data_folder(&self.global_messenger, path)
            {
                debug_log(format!("Serializing {:?}", path).as_str());
            }
        }
    }
}
