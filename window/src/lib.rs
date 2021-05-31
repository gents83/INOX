#![warn(clippy::all)]

use nrg_core::*;

mod config;
mod main_window;
mod rendering_system;
mod update_system;
mod window_system;

#[no_mangle]
pub extern "C" fn create_plugin() -> PluginHolder {
    let window = main_window::MainWindow::default();
    PluginHolder::new(Box::new(window))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin_holder: PluginHolder) {
    let boxed: Box<main_window::MainWindow> = plugin_holder.get_boxed_plugin();
    let window = *boxed;
    drop(window);
}
