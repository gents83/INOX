use super::super::handle::*;
use core::ffi::c_void;
use core::ptr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AndroidHandle {
    /// A pointer to an ANativeWindow.
    pub a_native_window: *mut c_void,
}

impl Handle for AndroidHandle {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}

impl TrustedHandle {
    pub fn new() -> TrustedHandle {
        AndroidHandle::empty().as_ref().clone()
    }
}

impl AsRef<TrustedHandle> for AndroidHandle {
    #[inline]
    fn as_ref(&self) -> &TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl AsMut<TrustedHandle> for AndroidHandle {
    #[inline]
    fn as_mut(&mut self) -> &mut TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl AndroidHandle {
    pub fn is_valid(&self) -> bool {
        !self.a_native_window.is_null()
    }
    pub fn empty() -> AndroidHandle {
        AndroidHandle {
            a_native_window: ptr::null_mut(),
        }
    }
}
