use std::sync::RwLock;

pub mod handle;
pub mod window;

pub static UI_VIEW: RwLock<*mut core::ffi::c_void> = RwLock::new(core::ptr::null_mut());

pub fn set_ui_view(view: *mut core::ffi::c_void) {
    *UI_VIEW.write().unwrap() = view;
}
