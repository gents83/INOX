#![warn(clippy::all)]

pub use crate::common::*;
pub use crate::fonts::*;

use nrg_core::*;
pub mod api {
    #[cfg(target_os = "ios")]
    #[path = "metal/backend.rs"]
    pub mod backend;

    //Vulkan is supported by Windows, Android, MacOs, Unix
    #[cfg(not(target_os = "ios"))]
    #[path = "vulkan/backend.rs"]
    pub mod backend;
}

pub mod common;
pub mod fonts;
mod voxels;

mod config;
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
