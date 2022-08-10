use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::super::handle::*;
use core::ffi::c_void;
use core::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    /// An Xlib `Window`.
    pub window: u32,
    /// A pointer to an Xlib `Display`.
    pub display: *mut c_void,
}

impl HandleImpl {
    pub fn as_raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = raw_window_handle::XlibHandle::empty();
        handle.window = self.window;
        handle.display = self.display;
        RawWindowHandle::Xlib(handle)
    }
    pub fn as_raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        let mut display_handle = XlibDisplayHandle::empty();
        display_handle.display = self.xconn.display as *mut _;
        display_handle.screen =
            unsafe { (self.xconn.xlib.XDefaultScreen)(self.xconn.display as *mut _) };
        RawDisplayHandle::Xlib(display_handle)
    }
    pub fn is_valid(&self) -> bool {
        !self.display.is_null()
    }
}

impl Handle for HandleImpl {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}
