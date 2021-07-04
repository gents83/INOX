use std::{
    fs::{copy, create_dir_all},
    path::{Path, PathBuf},
};

use nrg_messenger::{Message, MessengerRw};
use nrg_resources::{ResourceEvent, DATA_FOLDER, DATA_RAW_FOLDER};

pub fn convert_in_local_path(original_path: &Path, base_path: &Path) -> PathBuf {
    let path = original_path.to_str().unwrap().to_string();
    let path = path.replace(
        PathBuf::from(base_path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
        "",
    );
    let mut path = path.replace("\\", "/");
    if path.starts_with('/') {
        path.remove(0);
    }
    PathBuf::from(path)
}

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

pub fn copy_into_data_folder(global_messenger: &MessengerRw, path: &Path) -> bool {
    let mut from_source_to_compiled = path.to_str().unwrap().to_string();
    from_source_to_compiled = from_source_to_compiled.replace(
        PathBuf::from(DATA_RAW_FOLDER)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
        PathBuf::from(DATA_FOLDER)
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
            send_reloaded_event(global_messenger, new_path.as_path());
            return true;
        }
    }
    false
}

pub fn send_reloaded_event(messenger: &MessengerRw, new_path: &Path) {
    let dispatcher = messenger.read().unwrap().get_dispatcher();
    dispatcher
        .write()
        .unwrap()
        .send(ResourceEvent::Reload(new_path.to_path_buf()).as_boxed())
        .ok();
}
