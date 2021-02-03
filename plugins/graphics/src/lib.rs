#![warn(clippy::all)]

// Common 

use gfx::GfxPlugin;

pub use crate::api::data_formats::*;
pub use crate::api::device::*;
pub use crate::api::instance::*;
pub use crate::api::material::*;
pub use crate::api::mesh::*;
pub use crate::api::pipeline::*;
pub use crate::api::renderer::*;
pub use crate::api::shader::*;

//Modules
pub mod api
{
    pub mod data_formats;
    pub mod device;
    pub mod instance;
    pub mod material;
    pub mod mesh;
    pub mod pipeline;
    pub mod renderer;
    pub mod rasterizer;
    pub mod render_pass;
    pub mod shader;
    pub mod viewport;
    pub mod utils;

    #[cfg(target_os = "ios")]
    #[path = "metal/backend.rs"]
    pub mod backend;

    //Vulkan is supported by Windows, Android, MacOs, Unix
    #[cfg(not(target_os = "ios"))] 
    #[path = "vulkan/backend.rs"]
    pub mod backend;
}

mod voxels 
{
    mod block;
    mod chunk;
    mod world;
}

mod fonts
{
    mod font;
    mod geometry;
    mod glyph;
    mod hershey;
    mod raster;
    mod test;
}

mod gfx;
mod rendering_system;


#[no_mangle]
pub extern fn create_plugin() -> *mut dyn nrg_app::Plugin {
    let plugin = gfx::GfxPlugin::default();
    let boxed = Box::new(plugin);
    Box::into_raw(boxed)
}

#[no_mangle]
pub extern fn destroy_plugin(ptr: *mut dyn nrg_app::Plugin) {
    let boxed: Box<gfx::GfxPlugin> = unsafe { Box::from_raw( std::mem::transmute_copy(&ptr) ) };
    let plugin = *boxed;
    drop(plugin);
}
