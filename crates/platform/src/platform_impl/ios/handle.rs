#![cfg(target_os = "ios")]

use raw_window_handle::{
    DisplayHandle, HandleError, RawDisplayHandle, RawWindowHandle, UiKitDisplayHandle,
    UiKitWindowHandle, WindowHandle,
};
use std::ffi::c_void;
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub ui_window: *mut c_void,
    pub ui_view: *mut c_void,
    pub ui_view_controller: *mut c_void,
}

unsafe impl Send for HandleImpl {}
unsafe impl Sync for HandleImpl {}

impl HandleImpl {
    pub fn as_window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        if let Some(ptr) = NonNull::new(self.ui_view) {
            let handle = UiKitWindowHandle::new(ptr);
            unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::UiKit(handle))) }
        } else {
            Err(HandleError::Unavailable)
        }
    }
    #[inline]
    pub fn as_display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe {
            Ok(DisplayHandle::borrow_raw(RawDisplayHandle::UiKit(
                UiKitDisplayHandle::new(),
            )))
        }
    }
}
