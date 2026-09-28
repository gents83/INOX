use std::sync::Arc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::SharedDataRc;

use crate::launcher::Launcher;

static mut MESSAGE_HUB: Option<MessageHubRc> = None;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(msg: &str);

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

    let context = launcher.context();

    launcher.add_static_plugin(
        "inox_common_script",
        &context,
        Some(inox_common_script::static_plugin::create_plugin),
        Some(inox_common_script::static_plugin::load_config_plugin),
        Some(inox_common_script::static_plugin::prepare_plugin),
        Some(inox_common_script::static_plugin::unprepare_plugin),
        Some(inox_common_script::static_plugin::destroy_plugin),
    );

    launcher.add_static_plugin(
        "inox_viewer",
        &context,
        Some(inox_viewer::static_plugin::create_plugin),
        Some(inox_viewer::static_plugin::load_config_plugin),
        Some(inox_viewer::static_plugin::prepare_plugin),
        Some(inox_viewer::static_plugin::unprepare_plugin),
        Some(inox_viewer::static_plugin::destroy_plugin),
    );
}

fn signal_browser_smoke_value(name: &str, value: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let key = wasm_bindgen::JsValue::from_str(&format!("__inox_smoke_{name}"));
    let value = wasm_bindgen::JsValue::from_str(value);
    let _ = js_sys::Reflect::set(window.as_ref(), &key, &value);
}

fn signal_browser_smoke(name: &str) {
    signal_browser_smoke_value(name, "true");
}

pub fn main_update(launcher: Arc<Launcher>) {
    unsafe {
        MESSAGE_HUB = Some(launcher.message_hub());
    }
    let can_continue = launcher.update();
    signal_browser_smoke("frame");
    if let Some(window) = web_sys::window() {
        let key = wasm_bindgen::JsValue::from_str("__inox_smoke_frame_count");
        let current = js_sys::Reflect::get(window.as_ref(), &key)
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0);
        signal_browser_smoke_value("frame_count", &(current + 1.0).to_string());
    }
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

fn hook(info: &std::panic::PanicHookInfo) {
    hook_impl(info);
}

fn hook_impl(info: &std::panic::PanicHookInfo) {
    let mut msg = info.to_string();
    msg.push_str("\n\nStack:\n\n");
    let e = Error::new();
    let stack = e.stack();
    msg.push_str(&stack);
    msg.push_str("\n\n");
    console_error(&msg);
}

fn init_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        std::panic::set_hook(Box::new(hook));
    });
}

#[wasm_bindgen]
pub fn send_event_from_string(s: &str) {
    #[allow(static_mut_refs)]
    if let Some(message_hub) = unsafe { MESSAGE_HUB.as_mut() } {
        debug_log!("Received string to convert into event:\n{}", s);
        message_hub.send_from_string(s.to_string());
    }
}
