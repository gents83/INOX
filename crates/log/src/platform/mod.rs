#[cfg(all(target_os = "android", not(target_arch = "wasm32")))]
pub mod android;
#[cfg(all(target_os = "android", not(target_arch = "wasm32")))]
#[allow(unused_imports)]
pub use android::*;

#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub mod pc;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
#[allow(unused_imports)]
pub use pc::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
