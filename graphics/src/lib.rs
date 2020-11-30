
// Common 

pub use self::device::*;
pub use self::instance::*;
pub use self::viewport::*;
pub use self::rasterizer::*;

//Modules

pub mod device;
pub mod instance;
pub mod viewport;
pub mod rasterizer;

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