use raw_window_handle::RawWindowHandle;

use super::platform_impl::platform::handle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
