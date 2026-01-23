#![warn(clippy::all)]

// Common
pub use crate::file::*;
pub use crate::library::*;
pub use crate::utils::*;

//Modules
pub mod file;
pub mod library;
pub mod utils;

pub mod platform_impl {
    #[cfg(all(not(target_arch = "wasm32"), target_os = "android"))]
    #[path = "android/platform.rs"]
    pub mod platform;

    #[cfg(all(not(target_arch = "wasm32"), target_os = "ios"))]
    #[path = "ios/platform.rs"]
    pub mod platform;

    #[cfg(all(not(target_arch = "wasm32"), unix, not(target_os = "android"), not(target_os = "ios")))]
    #[path = "unix/platform.rs"]
    pub mod platform;

    #[cfg(target_arch = "wasm32")]
    #[path = "web/platform.rs"]
    pub mod platform;

    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    #[path = "windows/platform.rs"]
    pub mod platform;
}
