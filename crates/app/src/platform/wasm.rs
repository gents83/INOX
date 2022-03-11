#![cfg(target_arch = "wasm32")]

use std::sync::Arc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use inox_messenger::MessageHubRc;
use inox_profiler::debug_log;
use inox_resources::SharedDataRc;

use crate::launcher::Launcher;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(msg: String);

    type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(structural, method, getter)]
    fn stack(error: &Error) -> String;
}

pub fn setup_env() {
    init_panic_hook();
}

pub fn binarizer_start(_shared_data: SharedDataRc, _message_hub: MessageHubRc) {}
pub fn binarizer_update(_: ()) {}
pub fn binarizer_stop(_: ()) {}

pub fn load_plugins(launcher: &Arc<Launcher>) {
    debug_log!("Loading plugins");

    launcher.add_static_plugin(
        "inox_viewer",
        Some(|| inox_viewer::viewer::create_plugin()),
        Some(|app| inox_viewer::viewer::prepare_plugin(app)),
        Some(|app| inox_viewer::viewer::unprepare_plugin(app)),
        Some(|| inox_viewer::viewer::destroy_plugin()),
    );
}

pub fn main_update(launcher: Arc<Launcher>) {
    let can_continue = launcher.update();
    if can_continue {
        let cb = Closure::wrap(Box::new(move || {
            main_update(launcher.clone());
        }) as Box<dyn FnMut()>);
        web_sys::window()
            .unwrap()
            .request_animation_frame(cb.as_ref().unchecked_ref())
            .ok();
        cb.forget();
    }
}

fn hook(info: &std::panic::PanicInfo) {
    hook_impl(info);
}

fn hook_impl(info: &std::panic::PanicInfo) {
    let mut msg = info.to_string();
    msg.push_str("\n\nStack:\n\n");
    let e = Error::new();
    let stack = e.stack();
    msg.push_str(&stack);
    msg.push_str("\n\n");
    error(msg);
}

fn init_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        std::panic::set_hook(Box::new(hook));
    });
}

#[wasm_bindgen]
pub fn pass_command_line_parameters(s: &str) {
    debug_log!("Received command_line_parameters:\n{}", s);
}
