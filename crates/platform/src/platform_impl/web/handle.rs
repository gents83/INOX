use wasm_bindgen::JsCast;

use raw_window_handle::{
    DisplayHandle, RawDisplayHandle, RawWindowHandle, WebDisplayHandle, WebWindowHandle,
    WindowHandle,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleImpl {
    /// An ID value inserted into the data attributes of the canvas element as 'raw-handle'
    ///
    /// When accessing from JS, the attribute will automatically be called rawHandle
    ///
    /// Each canvas created by the windowing system should be assigned their own unique ID.
    /// 0 should be reserved for invalid / null IDs.
    pub id: u32,
}

impl HandleImpl {
    pub fn as_window_handle(&self) -> WindowHandle {
        let handle = WebWindowHandle::new(self.id);
        unsafe { WindowHandle::borrow_raw(RawWindowHandle::Web(handle)) }
    }
    #[inline]
    pub fn as_display_handle(&self) -> DisplayHandle {
        unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Web(WebDisplayHandle::new())) }
    }
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
    pub fn canvas(&self) -> web_sys::HtmlCanvasElement {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        canvas
    }
}
