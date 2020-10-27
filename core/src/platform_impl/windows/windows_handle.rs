use core::ffi::c_void;
use core::ptr;
use super::super::handle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowsHandle {
    /// A Win32 HWND handle.
    pub hwnd: *mut c_void,
    /// The HINSTANCE associated with this type's HWND.
    pub hinstance: *mut c_void,
}

impl Handle for WindowsHandle {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}

impl TrustedHandle {
    pub fn new() -> TrustedHandle {
        WindowsHandle::empty().as_ref().clone()
    }
}

impl AsRef<TrustedHandle> for WindowsHandle {
    #[inline]
    fn as_ref(&self) -> &TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl AsMut<TrustedHandle> for WindowsHandle {
    #[inline]
    fn as_mut(&mut self) -> &mut TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl WindowsHandle {
    pub fn is_valid(&self) -> bool {
        !self.hwnd.is_null()
    }
    pub fn empty() -> WindowsHandle {
        WindowsHandle {
            hwnd: ptr::null_mut(),
            hinstance: ptr::null_mut(),
        } 
    }
}