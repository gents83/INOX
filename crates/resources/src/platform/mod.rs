#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_os = "windows")]
pub use pc::*;
#[cfg(target_os = "windows")]
pub mod pc;
