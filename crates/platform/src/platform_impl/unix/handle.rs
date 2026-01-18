#![cfg(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly", target_os = "netbsd", target_os = "openbsd"))]

use raw_window_handle::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, XlibDisplayHandle,
    XlibWindowHandle, WindowHandle,
};
use std::ptr::NonNull;
use std::os::raw::{c_ulong, c_void};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub window: c_ulong,
    pub display: *mut c_void,
}

unsafe impl Send for HandleImpl {}
unsafe impl Sync for HandleImpl {}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle<'_> {
        let handle = XlibWindowHandle::new(self.window);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xlib(handle)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle<'_> {
        let display = NonNull::new(self.display);
        let handle = XlibDisplayHandle::new(display, 0);
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Xlib(handle)) }
    }
}
