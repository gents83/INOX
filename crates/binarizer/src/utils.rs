use std::{
    fs::{copy, create_dir_all},
    path::{Path, PathBuf},
};

use inox_filesystem::convert_in_local_path;
use inox_messenger::MessageHubRc;
use inox_resources::{Data, ReloadEvent};

pub fn need_to_binarize(original_path: &Path, new_path: &Path) -> bool {
    let mut need_copy = false;
    if let Ok(raw_time) = std::fs::metadata(original_path).unwrap().modified() {
        if !new_path.exists() {
            need_copy = true;
        } else if let Ok(data_time) = std::fs::metadata(new_path).unwrap().modified() {
            if data_time < raw_time {
                need_copy = true;
            }
        } else {
            need_copy = true;
        }
    }
    need_copy
}

pub fn copy_into_data_folder(message_hub: &MessageHubRc, path: &Path) -> bool {
    let mut from_source_to_compiled = path.to_str().unwrap().to_string();
    from_source_to_compiled = from_source_to_compiled.replace(
        Data::data_raw_folder()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
        Data::data_folder()
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
            send_reloaded_event(message_hub, new_path.as_path());
            return true;
        }
    }
    false
}

pub fn send_reloaded_event(message_hub: &MessageHubRc, new_path: &Path) {
    message_hub.send_event(ReloadEvent::Reload(new_path.to_path_buf()));
}

pub fn to_local_path(original_path: &Path) -> PathBuf {
    let base_path = convert_in_local_path(original_path, Data::data_raw_folder().as_path());
    let path = convert_in_local_path(base_path.as_path(), Data::data_folder().as_path());
    path
}
