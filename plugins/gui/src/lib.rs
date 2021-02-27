#![warn(clippy::all)]

use nrg_core::*;

mod colors;
mod config;
mod gui;
mod gui_updater;
mod widgets;

#[no_mangle]
pub extern "C" fn create_plugin() -> PluginHolder {
    let gui = gui::Gui::default();
    PluginHolder::new(Box::new(gui))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin_holder: PluginHolder) {
    let boxed: Box<gui::Gui> = plugin_holder.get_boxed_plugin();
    let gui = *boxed;
    drop(gui);
}
