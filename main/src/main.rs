use std::path::PathBuf;

use nrg_app::*;


fn main() {
    let mut app = App::new();

    let path = PathBuf::from("nrg_game.dll");
    let _game_plugin_id = app.add_plugin(path);
    
    app.run();
}