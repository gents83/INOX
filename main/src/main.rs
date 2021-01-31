use std::path::PathBuf;

use nrg_app::*;


fn main() {
    let mut app = App::new();

    {
        let path = PathBuf::from("nrg_window.dll");
        app.add_plugin(path);
    }

    {
        let path = PathBuf::from("nrg_game.dll");
        app.add_plugin(path);
    }
    
    app.run();
}