use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::super::handle::*;
use core::ffi::c_void;
use core::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    /// A pointer to an ANativeWindow.
    pub a_native_window: *mut c_void,
}

impl HandleImpl {
    pub fn as_raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = raw_window_handle::AndroidWindowHandle::empty();
        if let Some(native_window) = ndk_glue::native_window().as_ref() {
            handle.a_native_window = unsafe { native_window.ptr().as_mut() as *mut _ as *mut _ }
        } else {
            panic!("Cannot get the native window");
        };
        RawWindowHandle::AndroidNdk(handle)
    }
    #[inline]
    pub fn as_raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Android(raw_window_handle::AndroidDisplayHandle::empty())
    }
    pub fn is_valid(&self) -> bool {
        !self.a_native_window.is_null()
    }
}

impl Handle for HandleImpl {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}
