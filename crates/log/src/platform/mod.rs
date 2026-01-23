#[cfg(target_os = "android")]
pub use android::*;
#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub mod pc;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub use pc::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
