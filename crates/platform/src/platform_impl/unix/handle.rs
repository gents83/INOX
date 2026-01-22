use raw_window_handle::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, WindowHandle, XlibDisplayHandle,
    XlibWindowHandle,
};
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub window: std::os::raw::c_ulong,
    pub display: *mut std::ffi::c_void,
}

unsafe impl Send for HandleImpl {}
unsafe impl Sync for HandleImpl {}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle {
        let mut handle = XlibWindowHandle::new(self.window);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xlib(handle)) }
    }

    pub fn as_display_handle(&self) -> DisplayHandle {
        let display = NonNull::new(self.display).unwrap();
        let handle = XlibDisplayHandle::new(Some(display), 0);
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Xlib(handle)) }
    }
}
