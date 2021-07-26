use core::ffi::c_void;
use core::ptr;
use super::super::handle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacOSHandle {
    pub ns_window: *mut c_void,
    pub ns_view: *mut c_void,
}

impl Handle for MacOSHandle {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}

impl TrustedHandle {
    pub fn new() -> TrustedHandle {
        MacOSHandle::empty().as_ref().clone()
    }
}

impl AsRef<TrustedHandle> for MacOSHandle {
    #[inline]
    fn as_ref(&self) -> &TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl AsMut<TrustedHandle> for MacOSHandle {
    #[inline]
    fn as_mut(&mut self) -> &mut TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl MacOSHandle {
    pub fn is_valid(&self) -> bool {
        !self.ns_window.is_null()
    }
    pub fn empty() -> MacOSHandle {
        MacOSHandle {
            ns_window: ptr::null_mut(),
            ns_view: ptr::null_mut(),
        } 
    }
}