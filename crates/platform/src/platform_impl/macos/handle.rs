#![cfg(target_os = "macos")]

use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, DisplayHandle, HandleError, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};
use std::ffi::c_void;
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub ns_window: *mut c_void,
    pub ns_view: *mut c_void,
}

unsafe impl Send for HandleImpl {}
unsafe impl Sync for HandleImpl {}

impl HandleImpl {
    pub fn as_window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        if let Some(ptr) = NonNull::new(self.ns_view) {
            let handle = AppKitWindowHandle::new(ptr);
            unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle))) }
        } else {
            Err(HandleError::Unavailable)
        }
    }
    #[inline]
    pub fn as_display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe {
            Ok(DisplayHandle::borrow_raw(RawDisplayHandle::AppKit(
                AppKitDisplayHandle::new(),
            )))
        }
    }
}
