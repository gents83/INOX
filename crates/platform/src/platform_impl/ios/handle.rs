use raw_window_handle::{
    iKitDisplayHandle, DisplayHandle, RawDisplayHandle, URawWindowHandle, UiKitWindowHandle,
    WindowHandle,
};

use super::super::handle::*;
use core::ffi::c_void;
use core::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub ui_window: *mut c_void,
    pub ui_view: *mut c_void,
    pub ui_view_controller: *mut c_void,
}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle {
        let view = NonNull::from(self.ui_view as _).cast();
        let handle = UiKitWindowHandle::new(view);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::UiKit(handle)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle {
        let handle = UiKitDisplayHandle::new();
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::UiKit(handle)) }
    }
    pub fn is_valid(&self) -> bool {
        !self.ui_window.is_null()
    }
}

impl Handle for HandleImpl {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}
