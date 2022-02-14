#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

use inox_core::App;

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

pub fn binarizer_start(_app: &App) {}
pub fn binarizer_update(_: ()) {}
pub fn binarizer_stop(_: ()) {}

pub fn hook(info: &std::panic::PanicInfo) {
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

#[wasm_bindgen]
pub fn init_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        std::panic::set_hook(Box::new(hook));
    });
}
