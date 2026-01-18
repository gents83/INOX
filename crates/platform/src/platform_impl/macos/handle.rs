#![cfg(target_os = "macos")]

use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, DisplayHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};
use std::ptr::NonNull;
use std::ffi::c_void;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub ns_window: *mut c_void,
    pub ns_view: *mut c_void,
}

unsafe impl Send for HandleImpl {}
unsafe impl Sync for HandleImpl {}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle<'_> {
        let mut handle = AppKitWindowHandle::new(NonNull::new(self.ns_view).unwrap());
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle<'_> {
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::AppKit(AppKitDisplayHandle::new())) }
    }
}
