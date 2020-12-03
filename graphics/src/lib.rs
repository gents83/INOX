
// Common 

pub use self::instance::*;

//Modules

pub mod instance;
mod device;
mod viewport;
mod rasterizer;
mod data_formats;

pub mod api
{
    #[cfg(target_os = "ios")]
    #[path = "metal/backend.rs"]
    pub mod backend;

    //Vulkan is supported by Windows, Android, MacOs, Unix
    #[cfg(not(target_os = "ios"))] 
    #[path = "vulkan/backend.rs"]
    pub mod backend;
}