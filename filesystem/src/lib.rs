#![warn(clippy::all)]

// Common
pub use crate::library::*;
pub use crate::utils::*;

//Modules
pub mod library;
pub mod utils;

pub mod platform_impl {
    #[cfg(target_os = "android")]
    #[path = "android/platform.rs"]
    pub mod platform;

    #[cfg(target_os = "ios")]
    #[path = "ios/platform.rs"]
    pub mod platform;

    #[cfg(target_os = "macos")]
    #[path = "macos/platform.rs"]
    pub mod platform;

    #[cfg(target_os = "unix")]
    #[path = "unix/platform.rs"]
    pub mod platform;

    #[cfg(target_os = "wasm32")]
    #[path = "web/platform.rs"]
    pub mod platform;

    #[cfg(target_os = "windows")]
    #[path = "windows/platform.rs"]
    pub mod platform;
}
