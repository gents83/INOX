use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

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
    pub fn as_raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = raw_window_handle::UiKitHandle::empty();
        handle.ui_window = self.window as _;
        handle.ui_view = self.view as _;
        handle.ui_view_controller = self.view_controller as _;
        RawWindowHandle::UiKit(handle)
    }
    #[inline]
    pub fn as_raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::UiKit(raw_window_handle::UiKitDisplayHandle::empty())
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
