use core::ffi::c_void;
use std::ptr::NonNull;
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub a_native_window: *mut c_void,
}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle {
        let native_window = NonNull::new(self.a_native_window).unwrap();
        let handle = AndroidNdkWindowHandle::new(native_window);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(handle)) }
    }

    pub fn as_display_handle(&self) -> DisplayHandle {
        let handle = AndroidDisplayHandle::new();
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Android(handle)) }
    }

    pub fn is_valid(&self) -> bool {
        !self.a_native_window.is_null()
    }
}
