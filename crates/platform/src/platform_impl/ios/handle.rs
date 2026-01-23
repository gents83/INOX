use core::ffi::c_void;
use std::ptr::NonNull;
use raw_window_handle::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, UiKitDisplayHandle,
    UiKitWindowHandle, WindowHandle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandleImpl {
    pub ui_view: *mut c_void,
}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle<'_> {
        let view = NonNull::new(self.ui_view).unwrap();
        let handle = UiKitWindowHandle::new(view);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::UiKit(handle)) }
    }

    pub fn as_display_handle(&self) -> DisplayHandle<'_> {
        let handle = UiKitDisplayHandle::new();
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::UiKit(handle)) }
    }

    pub fn is_valid(&self) -> bool {
        !self.ui_view.is_null()
    }
}
