#![warn(clippy::all)]

// Common 

pub use self::data_formats::*;
pub use self::device::*;
pub use self::instance::*;
pub use self::material::*;
pub use self::mesh::*;
pub use self::pipeline::*;
pub use self::renderer::*;
pub use self::shader::*;

//Modules

mod data_formats;
mod device;
mod instance;
mod material;
mod mesh;
mod pipeline;
mod renderer;
mod rasterizer;
mod render_pass;
mod shader;
mod viewport;
mod utils;

pub mod voxels 
{
    pub mod block;
    pub mod chunk;
    pub mod world;
}

pub mod fonts
{
    pub mod font;
    pub mod geometry;
    pub mod glyph;
    pub mod hershey;
    pub mod raster;
    mod test;
}

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