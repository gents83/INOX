
use std::sync::RwLock;

pub mod handle;
pub mod window;
pub mod watcher;

pub struct UIViewWrapper(*mut core::ffi::c_void);
unsafe impl Send for UIViewWrapper {}
unsafe impl Sync for UIViewWrapper {}

pub static UI_VIEW: RwLock<UIViewWrapper> = RwLock::new(UIViewWrapper(core::ptr::null_mut()));

pub fn set_ui_view(view: *mut core::ffi::c_void) {
    *UI_VIEW.write().unwrap() = UIViewWrapper(view);
}
