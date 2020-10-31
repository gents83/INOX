use core::ffi::c_void;
use core::ptr;
use super::super::handle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IOSHandle {
    pub ui_window: *mut c_void,
    pub ui_view: *mut c_void,
    pub ui_view_controller: *mut c_void,
}

impl Handle for IOSHandle {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}

impl TrustedHandle {
    pub fn new() -> TrustedHandle {
        IOSHandle::empty().as_ref().clone()
    }
}

impl AsRef<TrustedHandle> for IOSHandle {
    #[inline]
    fn as_ref(&self) -> &TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl AsMut<TrustedHandle> for IOSHandle {
    #[inline]
    fn as_mut(&mut self) -> &mut TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl IOSHandle {
    pub fn is_valid(&self) -> bool {
        !self.ui_window.is_null()
    }
    pub fn empty() -> IOSHandle {
        IOSHandle {
            ui_window: ptr::null_mut(),
            ui_view: ptr::null_mut(),
            ui_view_controller: ptr::null_mut(),
        } 
    }
}