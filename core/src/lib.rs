

// Common 

pub use self::platform_impl::platform::*;
pub use self::platform_impl::handle::*;
pub use self::platform_impl::window::*;

pub use self::entity::*;
pub use self::component::*;
pub use self::system::*;

//Modules

pub mod platform_impl
{
    pub mod handle;
    pub mod window;

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

pub mod entity;
pub mod component;
pub mod system;
