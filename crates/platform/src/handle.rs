use raw_window_handle::{DisplayHandle, WindowHandle, HandleError};

use super::platform_impl::platform::handle::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle {
    pub handle_impl: HandleImpl,
}

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

impl raw_window_handle::HasWindowHandle for Handle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Ok(self.handle_impl.as_window_handle())
    }
}

impl raw_window_handle::HasDisplayHandle for Handle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(self.handle_impl.as_display_handle())
    }
}
