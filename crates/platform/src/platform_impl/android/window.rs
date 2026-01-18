#![cfg(target_os = "android")]

use crate::handle::{Handle, HandleImpl};
use crate::window::{Window, WindowEvent};
use crate::{Key, MouseButton, MouseState, InputState, KeyEvent, KeyTextEvent, MouseEvent};
use inox_messenger::MessageHubRc;
use std::path::Path;
use std::ptr;

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
                window: ptr::null_mut(),
            }
        }
    }

    pub fn change_title(_handle: &Handle, _title: &str) {}
    pub fn change_visibility(_handle: &Handle, _is_visible: bool) {}
    pub fn change_position(_handle: &Handle, _x: u32, _y: u32) {}
    pub fn change_size(_handle: &Handle, _width: u32, _height: u32) {}
    pub fn internal_update(_handle: &Handle) -> bool { true }
}

pub fn convert_key(_key: i32) -> Key {
    Key::Unidentified
}
