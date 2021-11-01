use super::super::handle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebHandle {
    /// An ID value inserted into the data attributes of the canvas element as 'raw-handle'
    ///
    /// When accessing from JS, the attribute will automatically be called rawHandle
    ///
    /// Each canvas created by the windowing system should be assigned their own unique ID.
    /// 0 should be reserved for invalid / null IDs.
    pub id: u32,
}

impl Handle for WebHandle {
    fn is_valid(&self) -> bool {
        self.is_valid()
    }
}

impl TrustedHandle {
    pub fn new() -> TrustedHandle {
        WebHandle::empty().as_ref().clone()
    }
}

impl AsRef<TrustedHandle> for WebHandle {
    #[inline]
    fn as_ref(&self) -> &TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl AsMut<TrustedHandle> for WebHandle {
    #[inline]
    fn as_mut(&mut self) -> &mut TrustedHandle {
        unsafe { ::std::mem::transmute(self) }
    }
}

impl WebHandle {
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
    pub fn empty() -> WebHandle {
        WebHandle {
            id: ptr::null_mut(),
        }
    }
}
