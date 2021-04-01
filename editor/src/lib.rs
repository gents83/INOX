#![warn(clippy::all)]

use nrg_core::*;

mod config;
mod editor;
mod editor_updater;
mod widgets;

#[no_mangle]
pub extern "C" fn create_plugin() -> PluginHolder {
    let editor = editor::Editor::default();
    PluginHolder::new(Box::new(editor))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin_holder: PluginHolder) {
    let boxed: Box<editor::Editor> = plugin_holder.get_boxed_plugin();
    let editor = *boxed;
    drop(editor);
}
