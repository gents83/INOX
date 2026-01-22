#![warn(clippy::all)]
#![allow(clippy::too_many_arguments)]

// Common
pub use self::handle::*;
pub use self::input::*;
pub use self::thread::*;
pub use self::watcher::*;
pub use self::window::*;

//Modules
pub mod handle;
pub mod thread;
pub mod watcher;
pub mod window;

pub mod input;

pub type PlatformType = usize;
pub const PLATFORM_TYPE_PC: PlatformType = 0;
pub const PLATFORM_TYPE_WEB: PlatformType = 1;
pub const PLATFORM_TYPE_ANDROID: PlatformType = 2;
pub const PLATFORM_TYPE_IOS: PlatformType = 3;

pub const PLATFORM_TYPE_PC_NAME: &str = "pc";
pub const PLATFORM_TYPE_WEB_NAME: &str = "web";
pub const PLATFORM_TYPE_ANDROID_NAME: &str = "android";
pub const PLATFORM_TYPE_IOS_NAME: &str = "ios";

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

    #[cfg(target_os = "linux")]
    #[path = "unix/platform.rs"]
    pub mod platform;

    #[cfg(target_arch = "wasm32")]
    #[path = "web/platform.rs"]
    pub mod platform;

    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    #[path = "windows/platform.rs"]
    pub mod platform;
}
