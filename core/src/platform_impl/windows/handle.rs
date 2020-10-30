use super::super::handle::*;
use super::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    /// A Win32 HWND handle.
    pub hwnd: HWND,
    /// The HINSTANCE associated with this type's HWND.
    pub hinstance: HINSTANCE,
}

impl AsRef<Handle> for HandleImpl {
    #[inline]
    fn as_ref(&self) -> &Handle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl AsMut<Handle> for HandleImpl {
    #[inline]
    fn as_mut(&mut self) -> &mut Handle {
        unsafe { ::std::mem::transmute(self) }
    }
}