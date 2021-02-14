#![warn(clippy::all)]

use nrg_core::*;

// Common

pub use crate::api::data_formats::*;
pub use crate::api::device::*;
pub use crate::api::instance::*;
pub use crate::api::material::*;
pub use crate::api::mesh::*;
pub use crate::api::pipeline::*;
pub use crate::api::renderer::*;
pub use crate::api::shader::*;

pub use crate::fonts::font::*;

//Modules
pub mod api {
    pub mod data_formats;
    pub mod device;
    pub mod instance;
    pub mod material;
    pub mod mesh;
    pub mod pipeline;
    pub mod rasterizer;
    pub mod render_pass;
    pub mod renderer;
    pub mod shader;
    pub mod utils;
    pub mod viewport;

    #[cfg(target_os = "ios")]
    #[path = "metal/backend.rs"]
    pub mod backend;

    //Vulkan is supported by Windows, Android, MacOs, Unix
    #[cfg(not(target_os = "ios"))]
    #[path = "vulkan/backend.rs"]
    pub mod backend;
}

mod voxels {
    pub mod block;
    pub mod chunk;
    pub mod world;
}

pub mod fonts {
    pub mod font;
    mod geometry;
    mod glyph;
    mod hershey;
    mod raster;
    mod test;
}

mod gfx;
mod rendering_system;

#[no_mangle]
pub extern "C" fn create_plugin() -> PluginHolder {
    let plugin = gfx::GfxPlugin::default();
    PluginHolder::new(Box::new(plugin))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin_holder: PluginHolder) {
    let boxed: Box<gfx::GfxPlugin> = plugin_holder.get_boxed_plugin();
    let plugin = *boxed;
    drop(plugin);
}
