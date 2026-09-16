use raw_window_handle::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, WebDisplayHandle, WebWindowHandle,
    WindowHandle,
};

pub struct HandleImpl {
    /// An ID value inserted into the data attributes of the canvas element as 'raw-handle'
    ///
    /// When accessing from JS, the attribute will automatically be called rawHandle
    ///
    /// Each canvas created by the windowing system should be assigned their own unique ID.
    /// 0 should be reserved for invalid / null IDs.
    pub id: u32,
    pub(crate) canvas: web_sys::HtmlCanvasElement,
}

impl Clone for HandleImpl {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            canvas: self.canvas.clone(),
        }
    }
}

impl std::fmt::Debug for HandleImpl {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HandleImpl")
            .field("id", &self.id)
            .finish()
    }
}

impl PartialEq for HandleImpl {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for HandleImpl {}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle<'_> {
        let handle = WebWindowHandle::new(self.id);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::Web(handle)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle<'_> {
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Web(WebDisplayHandle::new())) }
    }
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
    pub fn canvas(&self) -> web_sys::HtmlCanvasElement {
        self.canvas.clone()
    }
}
