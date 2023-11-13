use raw_window_handle::{DisplayHandle, WindowHandle, RawWindowHandle, RawDisplayHandle, XlibWindowHandle, XlibDisplayHandle};

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
    pub fn as_window_handle(&self) -> WindowHandle {
        let mut window = XlibWindowHandle::new(self.window);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xlib(handle as _)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle {
        let display = NonNull::from(self.xconn.display as _).cast();
        let handle = XlibDisplayHandle::new(Some(display), 0 as _);
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Xlib(handle)) }
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
