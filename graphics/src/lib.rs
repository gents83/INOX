
// Common 

pub use self::device::*;
pub use self::instance::*;
pub use self::data_formats::*;
pub use self::material::*;
pub use self::mesh::*;
pub use self::renderer::*;

//Modules

mod instance;
mod device;
mod material;
mod mesh;
mod renderer;
mod viewport;
mod rasterizer;
mod render_pass;
mod data_formats;
mod utils;

mod block;
mod chunk;
mod world;

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