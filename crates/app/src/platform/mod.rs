#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use pc::*;
#[cfg(not(target_arch = "wasm32"))]
pub mod pc;
