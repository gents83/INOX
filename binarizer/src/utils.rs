use std::path::Path;

use nrg_messenger::{Message, MessengerRw};
use nrg_resources::ResourceEvent;

pub fn need_to_binarize(original_path: &Path, new_path: &Path) -> bool {
    let mut need_copy = false;
    if let Ok(raw_time) = std::fs::metadata(original_path).unwrap().modified() {
        if let Ok(data_time) = std::fs::metadata(new_path).unwrap().modified() {
            if data_time < raw_time {
                need_copy = true;
            }
        } else {
            need_copy = true;
        }
    }
    need_copy
}

pub fn send_reloaded_event(messenger: &MessengerRw, new_path: &Path) {
    let dispatcher = messenger.read().unwrap().get_dispatcher();
    dispatcher
        .write()
        .unwrap()
        .send(ResourceEvent::Reload(new_path.to_path_buf()).as_boxed())
        .ok();
}
