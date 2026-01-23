use std::path::Path;
use android_activity::{MainEvent, PollEvent};

use inox_messenger::MessageHubRc;

use super::handle::*;
use super::{ANDROID_APP, NATIVE_WINDOW, NativeWindowWrapper};
use crate::handle::*;
use crate::window::*;

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
        let app = ANDROID_APP.get().expect("AndroidApp not initialized");

        // Wait for window creation
        let window_ptr;
        loop {
            let win = NATIVE_WINDOW.read().unwrap();
            if !win.0.is_null() {
                window_ptr = win.0;
                break;
            }
            drop(win);

            app.poll_events(Some(std::time::Duration::from_millis(10)), |event| {
                if let PollEvent::Main(MainEvent::InitWindow { .. }) = event {
                    let ptr = app.native_window().unwrap().ptr().as_ptr();
                    *NATIVE_WINDOW.write().unwrap() = NativeWindowWrapper(ptr as _);
                    *scale_factor = 1.0; // TODO: Get density
                                         // Update width/height?
                }
            });
        }

        Handle {
            handle_impl: HandleImpl {
                a_native_window: window_ptr,
            },
        }
    }

    pub fn change_title(_handle: &Handle, _title: &str) {}
    pub fn change_visibility(_handle: &Handle, _is_visible: bool) {}
    pub fn change_position(_handle: &Handle, _x: u32, _y: u32) {}
    pub fn change_size(_handle: &Handle, _width: u32, _height: u32) {}

    #[inline]
    pub fn internal_update(_handle: &Handle) -> bool {
        let app = ANDROID_APP.get().unwrap();
        let mut can_continue = true;
        app.poll_events(Some(std::time::Duration::from_millis(0)), |event| {
            match event {
                PollEvent::Main(MainEvent::Destroy) => {
                    can_continue = false;
                }
                PollEvent::Main(MainEvent::InitWindow { .. }) => {
                     *NATIVE_WINDOW.write().unwrap() = NativeWindowWrapper(app.native_window().unwrap().ptr().as_ptr() as _);
                }
                PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                     *NATIVE_WINDOW.write().unwrap() = NativeWindowWrapper(std::ptr::null_mut());
                }
                // TODO: Handle Input events
                _ => {}
            }
        });
        can_continue
    }
}
