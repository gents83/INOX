#![warn(clippy::all)]

mod game;
mod system;


#[no_mangle]
pub extern "C" fn create_plugin() -> *mut nrg_app::Plugin {
    let game = game::Game::default();
    let boxed = Box::new(game);
    Box::into_raw(boxed)
}