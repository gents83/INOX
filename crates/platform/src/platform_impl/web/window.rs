use inox_messenger::MessageHubRc;
use std::path::Path;

use wasm_bindgen::JsCast;

use super::handle::*;
use crate::handle::*;
use crate::window::*;

impl Window {
    pub fn create_handle(
        _title: String,
        _x: u32,
        _y: u32,
        _width: &mut u32,
        _height: &mut u32,
        _scale_factor: &mut f32,
        _icon_path: &Path,
        _events_dispatcher: &MessageHubRc,
    ) -> Handle {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        canvas.set_attribute("data-raw-handle", "0");
        let _canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        Handle {
            handle_impl: HandleImpl { id: 0 },
        }
    }

    pub fn change_title(_handle: &Handle, _title: &str) {}
    pub fn change_visibility(_handle: &Handle, _is_visible: bool) {}

    pub fn change_position(_handle: &Handle, _x: u32, _y: u32) {}

    pub fn change_size(_handle: &Handle, _width: u32, _height: u32) {}

    #[inline]
    pub fn internal_update(_handle: &Handle) -> bool {
        true
    }
}
