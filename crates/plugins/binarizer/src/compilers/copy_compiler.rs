use std::path::{Path, PathBuf};

use crate::{copy_into_data_folder, ExtensionHandler};
use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_nodes::NodeTree;
use inox_render::{ComputePipeline, Light, Material, Mesh, RenderPipeline};
use inox_resources::SerializableResource;
use inox_scene::{Camera, Object, Scene, Script};
use inox_serialize::SerializeFile;

const CONFIG_EXTENSION: &str = "cfg";

pub struct CopyCompiler {
    message_hub: MessageHubRc,
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
}

impl CopyCompiler {
    pub fn new(message_hub: MessageHubRc, data_raw_folder: &Path, data_folder: &Path) -> Self {
        Self {
            message_hub,
            data_raw_folder: data_raw_folder.to_path_buf(),
            data_folder: data_folder.to_path_buf(),
        }
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
                || ext.as_str() == RenderPipeline::extension()
                || ext.as_str() == ComputePipeline::extension()
                || ext.as_str() == Scene::extension()
                || ext.as_str() == Object::extension()
                || ext.as_str() == Camera::extension()
                || ext.as_str() == Light::extension()
                || ext.as_str() == Script::extension())
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
