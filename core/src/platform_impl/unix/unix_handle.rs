use core::ffi::c_void;
use core::ptr;
use super::super::handle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XlibHandle {
    /// An Xlib `Window`.
    pub window: u32,
    /// A pointer to an Xlib `Display`.
    pub display: *mut c_void,
}

impl Handle for XlibHandle {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}

impl TrustedHandle {
    pub fn new() -> TrustedHandle {
        XlibHandle::empty().as_ref().clone()
    }
}

impl AsRef<TrustedHandle> for XlibHandle {
    #[inline]
    fn as_ref(&self) -> &TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl AsMut<TrustedHandle> for XlibHandle {
    #[inline]
    fn as_mut(&mut self) -> &mut TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl XlibHandle {
    pub fn is_valid(&self) -> bool {
        !self.display.is_null()
    }
    pub fn empty() -> XlibHandle {
        XlibHandle {
            window: 0,
            display: ptr::null_mut(),
        } 
    }
}