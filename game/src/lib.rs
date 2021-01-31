#![warn(clippy::all)]

mod game;
mod system;


#[no_mangle]
pub extern fn create_plugin() -> *mut dyn nrg_app::Plugin {
    let game = game::Game::default();
    let boxed = Box::new(game);
    Box::into_raw(boxed)
}
