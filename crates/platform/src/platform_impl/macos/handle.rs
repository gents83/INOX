use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, DisplayHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};

use super::super::handle::*;
use core::ffi::c_void;
use core::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub ns_window: *mut c_void,
    pub ns_view: *mut c_void,
}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle {
        let view = NonNull::from(self.ns_view as _).cast();
        let handle = XlibWindowHandle::new(view);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle {
        let handle = AppKitDisplayHandle::new();
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::AppKit(handle)) }
    }
    pub fn is_valid(&self) -> bool {
        !self.ns_window.is_null()
    }
}

impl Handle for HandleImpl {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}
