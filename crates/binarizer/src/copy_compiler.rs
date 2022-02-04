use std::path::Path;

use crate::{copy_into_data_folder, ExtensionHandler};
use sabi_graphics::{Light, Material, Mesh, Pipeline};
use sabi_messenger::MessageHubRc;
use sabi_nodes::NodeTree;
use sabi_profiler::debug_log;
use sabi_resources::SerializableResource;
use sabi_scene::{Camera, Object, Scene, Script};
use sabi_serialize::SerializeFile;

const CONFIG_EXTENSION: &str = "cfg";

pub struct CopyCompiler {
    message_hub: MessageHubRc,
}

impl CopyCompiler {
    pub fn new(message_hub: MessageHubRc) -> Self {
        Self { message_hub }
    }
}

impl ExtensionHandler for CopyCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap().to_string();
            if (ext.as_str() == CONFIG_EXTENSION
                || ext.as_str() == NodeTree::extension()
                || ext.as_str() == Material::extension()
                || ext.as_str() == Mesh::extension()
                || ext.as_str() == Pipeline::extension()
                || ext.as_str() == Scene::extension()
                || ext.as_str() == Object::extension()
                || ext.as_str() == Camera::extension()
                || ext.as_str() == Light::extension()
                || ext.as_str() == Script::extension())
                && copy_into_data_folder(&self.message_hub, path)
            {
                debug_log(format!("Serializing {:?}", path).as_str());
            }
        }
    }
}
