#![warn(clippy::all)]
#![allow(dead_code)]

pub use inox_filesystem::*;

pub use self::macros::*;
pub mod macros;

#[cfg(debug_assertions)]
pub use self::profiler::*;

#[cfg(debug_assertions)]
pub mod profiler;

//Using Chrome browser for profiling
//https://www.chromium.org/developers/how-tos/trace-event-profiling-tool
//go to chrome://tracing and click on "Load"

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
