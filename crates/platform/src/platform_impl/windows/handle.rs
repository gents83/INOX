use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    /// A Win32 HWND handle.
    pub hwnd: HWND,
    /// The HINSTANCE associated with this type's HWND.
    pub hinstance: HINSTANCE,
}

impl HandleImpl {
    pub fn as_raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = raw_window_handle::Win32WindowHandle::empty();
        handle.hwnd = self.hwnd as *mut _;
        handle.hinstance = self.hinstance as *mut _;
        RawWindowHandle::Win32(handle)
    }
    #[inline]
    pub fn as_raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Windows(raw_window_handle::WindowsDisplayHandle::empty())
    }
}
