use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleImpl {
    /// An ID value inserted into the data attributes of the canvas element as 'raw-handle'
    ///
    /// When accessing from JS, the attribute will automatically be called rawHandle
    ///
    /// Each canvas created by the windowing system should be assigned their own unique ID.
    /// 0 should be reserved for invalid / null IDs.
    pub id: u32,
    pub canvas: web_sys::HtmlCanvasElement,
}

impl HandleImpl {
    pub fn as_raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = raw_window_handle::WebWindowHandle::empty();
        handle.id = self.id;
        RawWindowHandle::Web(handle)
    }
    #[inline]
    pub fn as_raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Web(raw_window_handle::WebDisplayHandle::empty())
    }
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
}
