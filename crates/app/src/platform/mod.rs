#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub use pc::*;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub mod pc;
