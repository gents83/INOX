use inox_messenger::MessageHubRc;
use raw_window_handle::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, WindowHandle, XlibDisplayHandle,
    XlibWindowHandle,
};
use std::ptr::NonNull;
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct HandleImpl {
    pub window: std::os::raw::c_ulong,
    pub display: *mut std::ffi::c_void,
    pub wm_delete_window: std::os::raw::c_ulong,
    pub wm_protocols: std::os::raw::c_ulong,
    pub events_dispatcher: MessageHubRc,
}

unsafe impl Send for HandleImpl {}
unsafe impl Sync for HandleImpl {}

impl fmt::Debug for HandleImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandleImpl")
            .field("window", &self.window)
            .field("display", &self.display)
            .finish()
    }
}

impl PartialEq for HandleImpl {
    fn eq(&self, other: &Self) -> bool {
        self.window == other.window && self.display == other.display && Arc::ptr_eq(&self.events_dispatcher, &other.events_dispatcher)
    }
}

impl Eq for HandleImpl {}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle<'_> {
        let handle = XlibWindowHandle::new(self.window);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xlib(handle)) }
    }

    pub fn as_display_handle(&self) -> DisplayHandle<'_> {
        let display = NonNull::new(self.display).unwrap();
        let handle = XlibDisplayHandle::new(Some(display), 0);
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Xlib(handle)) }
    }
}
