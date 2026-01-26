use std::path::Path;
use inox_messenger::MessageHubRc;
use super::handle::HandleImpl;
use crate::handle::Handle;
use crate::window::*;
use std::ptr::null_mut;

impl Window {
    pub fn create_handle(
        _title: String,
        _x: u32,
        _y: u32,
        _width: &mut u32,
        _height: &mut u32,
        scale_factor: &mut f32,
        _icon_path: &Path,
        _events_dispatcher: &MessageHubRc,
    ) -> Handle {
        *scale_factor = 1.0;
        Handle {
            handle_impl: HandleImpl {
                ns_window: null_mut(),
                ns_view: null_mut(),
            },
        }
    }

    pub fn change_title(_handle: &Handle, _title: &str) {}
    pub fn change_visibility(_handle: &Handle, _is_visible: bool) {}
    pub fn change_position(_handle: &Handle, _x: u32, _y: u32) {}
    pub fn change_size(_handle: &Handle, _width: u32, _height: u32) {}

    #[inline]
    pub fn internal_update(_handle: &Handle) -> bool {
        true
    }
}
