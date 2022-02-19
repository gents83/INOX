use js_sys::{Function, Map, Object, Reflect, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use wasm_bindgen_futures::{spawn_local, JsFuture};

pub struct Library {}
impl Library {
    #[inline]
    pub fn load(filename: &str) -> Self {
        let w = web_sys::window().unwrap().get(filename).unwrap();
        let f = Reflect::get(w.as_ref(), &"create_plugin".into())
            .unwrap()
            .dyn_into::<Function>()
            .expect("create_plugin export wasn't a function");
        Self {}
    }

    #[inline]
    pub fn get<T>(&self, symbol: &str) -> Option<T> {
        None
    }

    #[inline]
    pub fn close(&mut self) {}
}
