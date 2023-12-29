#![warn(clippy::all)]
#![allow(dead_code)]

#[cfg(feature = "superluminal")]
pub use self::superluminal::*;
pub mod superluminal;

#[cfg(feature = "chrometrace")]
pub use self::chrometrace::*;
pub mod chrometrace;

pub mod cpu_profiler;
pub mod gpu_profiler;

pub use self::cpu_profiler::*;
pub use self::gpu_profiler::*;

pub mod macros;

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

pub fn current_time_in_micros() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros()
        .try_into()
        .unwrap()
}
