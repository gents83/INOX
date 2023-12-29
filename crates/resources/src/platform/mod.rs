#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_os = "windows")]
pub mod pc;
