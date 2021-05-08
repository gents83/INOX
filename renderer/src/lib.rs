#![warn(clippy::all)]

use nrg_core::*;

mod config;
mod gfx;
mod rendering_system;
mod update_system;

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
