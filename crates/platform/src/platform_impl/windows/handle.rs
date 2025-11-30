use std::num::NonZeroIsize;

use raw_window_handle::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowHandle,
    WindowsDisplayHandle,
};

use super::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    /// A Win32 HWND handle.
    pub hwnd: HWND,
    /// The HINSTANCE associated with this type's HWND.
    pub hinstance: HINSTANCE,
}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle<'_> {
        let mut handle = Win32WindowHandle::new(NonZeroIsize::new(self.hwnd as _).unwrap());
        // Optionally set the GWLP_HINSTANCE.
        let hinstance = NonZeroIsize::new(self.hinstance as _).unwrap();
        handle.hinstance = Some(hinstance);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(handle)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle<'_> {
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Windows(WindowsDisplayHandle::new())) }
    }
}
