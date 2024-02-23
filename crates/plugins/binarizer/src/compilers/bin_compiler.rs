use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use crate::{need_to_binarize, send_reloaded_event, ExtensionHandler};
use inox_filesystem::File;
use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;
use inox_serialize::serialize_to_file;

const CONFIG_EXTENSION: &str = "cfg";

pub struct BinCompiler {
    message_hub: MessageHubRc,
    shared_data: SharedDataRc,
    data_raw_folder: PathBuf,
    data_folder: PathBuf,
}

impl BinCompiler {
    pub fn new(
        message_hub: MessageHubRc,
        shared_data: SharedDataRc,
        data_raw_folder: &Path,
        data_folder: &Path,
    ) -> Self {
        Self {
            message_hub,
            shared_data,
            data_raw_folder: data_raw_folder.to_path_buf(),
            data_folder: data_folder.to_path_buf(),
        }
    }
}

impl ExtensionHandler for BinCompiler {
    fn on_changed(&mut self, path: &Path) {
        if let Some(ext) = path.extension() {
            let ext = ext.to_str().unwrap().to_string();
            if ext.as_str() == CONFIG_EXTENSION {
                let mut from_source_to_compiled = path.to_str().unwrap().to_string();
                from_source_to_compiled = from_source_to_compiled.replace(
                    self.data_raw_folder
                        .canonicalize()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    self.data_folder.canonicalize().unwrap().to_str().unwrap(),
                );
                let new_path = PathBuf::from(from_source_to_compiled);
                if !new_path.exists() {
                    let result = create_dir_all(new_path.parent().unwrap());
                    debug_assert!(result.is_ok());
                }
                if need_to_binarize(path, new_path.as_path()) {
                    let mut file = File::new(&path);
                    if file.exists() {
                        let path = path.to_path_buf();
                        file.load(move |bytes| match serde_json::from_slice::<&T>(&bytes) {
                            Ok(data) => {
                                send_reloaded_event(&self.message_hub, new_path.as_path());
                                debug_log!("Serializing {:?}", path);
                                serialize_to_file(
                                    data,
                                    &new_path,
                                    self.shared_data.serializable_registry(),
                                )
                            }
                            Err(e) => {
                                eprintln!("Error {} - Unable to deserialize", e,);
                            }
                        });
                    }
                }
            }
        }
    }
}
