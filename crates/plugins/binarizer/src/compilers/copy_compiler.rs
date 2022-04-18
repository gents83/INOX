use std::path::Path;

use crate::{copy_into_data_folder, ExtensionHandler};
use inox_graphics::{Light, Material, Mesh, Pipeline};
use inox_messenger::MessageHubRc;
use inox_nodes::NodeTree;
use inox_log::debug_log;
use inox_resources::SerializableResource;
use inox_scene::{Camera, Object, Scene, Script};
use inox_serialize::SerializeFile;

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
                debug_log!("Serializing {:?}", path);
            }
        }
    }
}
