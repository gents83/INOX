

// Common 
pub use self::handle::*;
pub use self::window::*;

//Modules
pub mod handle;
pub mod window;

pub mod platform_impl
{
    #[cfg(target_os = "android")]
    #[path = "android/platform.rs"]
    pub mod platform;

    #[cfg(target_os = "ios")]
    #[path = "iosmacos/platform.rs"]
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
