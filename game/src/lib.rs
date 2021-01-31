#![warn(clippy::all)]

mod game;
mod system;


#[no_mangle]
pub extern fn create_plugin() -> *mut dyn nrg_app::Plugin {
    let game = game::Game::default();
    let boxed = Box::new(game);
    Box::into_raw(boxed)
}

#[no_mangle]
pub extern fn destroy_plugin(ptr: *mut dyn nrg_app::Plugin) {
    let boxed: Box<game::Game> = unsafe { Box::from_raw( std::mem::transmute_copy(&ptr) ) };
    let game = *boxed;
    drop(game);
}
