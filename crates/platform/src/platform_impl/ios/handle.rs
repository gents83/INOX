#![cfg(target_os = "ios")]

use raw_window_handle::{
    UiKitDisplayHandle, UiKitWindowHandle, DisplayHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};
use std::ptr::NonNull;
use std::ffi::c_void;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub ui_window: *mut c_void,
    pub ui_view: *mut c_void,
    pub ui_view_controller: *mut c_void,
}

unsafe impl Send for HandleImpl {}
unsafe impl Sync for HandleImpl {}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle<'_> {
        let mut handle = UiKitWindowHandle::new(NonNull::new(self.ui_view).unwrap());
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::UiKit(handle)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle<'_> {
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::UiKit(UiKitDisplayHandle::new())) }
    }
}
