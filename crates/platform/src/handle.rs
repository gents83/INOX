use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::platform_impl::platform::handle::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle {
    pub handle_impl: HandleImpl,
}

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

unsafe impl raw_window_handle::HasRawWindowHandle for Handle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.handle_impl.as_raw_window_handle()
    }
}

unsafe impl raw_window_handle::HasRawDisplayHandle for Handle {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.handle_impl.as_raw_display_handle()
    }
}
