use std::path::Path;
use inox_messenger::MessageHubRc;
use crate::handle::*;
use crate::window::*;
use super::handle::HandleImpl;

impl Window {
    pub fn create_handle(
        _title: String,
        _x: u32,
        _y: u32,
        _width: &mut u32,
        _height: &mut u32,
        _scale_factor: &mut f32,
        _icon_path: &Path,
        _events_dispatcher: &MessageHubRc,
    ) -> Handle {
        Handle {
            handle_impl: HandleImpl {
                window: std::ptr::null_mut(),
                display: std::ptr::null_mut(),
            },
        }
    }

    pub fn change_title(_handle: &Handle, _title: &str) {
    }
    pub fn change_visibility(_handle: &Handle, _is_visible: bool) {
    }

    pub fn change_position(_handle: &Handle, _x: u32, _y: u32) {
    }

    pub fn change_size(_handle: &Handle, _width: u32, _height: u32) {
    }

    #[inline]
    pub fn internal_update(_handle: &Handle) -> bool {
        true
    }
}
