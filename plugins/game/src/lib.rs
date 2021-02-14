#![warn(clippy::all)]

use nrg_core::*;

mod config;
mod game;
mod system;

#[no_mangle]
pub extern "C" fn create_plugin() -> PluginHolder {
    let game = game::Game::default();
    PluginHolder::new(Box::new(game))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin_holder: PluginHolder) {
    let boxed: Box<game::Game> = plugin_holder.get_boxed_plugin();
    let game = *boxed;
    drop(game);
}
