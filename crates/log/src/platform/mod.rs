#[cfg(target_os = "android")]
pub mod android;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub mod pc;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
