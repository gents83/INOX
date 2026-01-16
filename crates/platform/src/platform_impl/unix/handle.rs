use raw_window_handle::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, WindowHandle, XlibDisplayHandle,
    XlibWindowHandle,
};

use core::ffi::c_void;
use core::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub window: *mut c_void,
    pub display: *mut c_void,
}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle<'_> {
        let window = XlibWindowHandle::new(self.window as _);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xlib(window)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle<'_> {
        let display = NonNull::new(self.display).unwrap();
        let handle = XlibDisplayHandle::new(Some(display), 0);
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Xlib(handle)) }
    }
    pub fn is_valid(&self) -> bool {
        !self.display.is_null()
    }
}
