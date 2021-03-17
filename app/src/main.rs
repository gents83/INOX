use std::path::PathBuf;

use nrg_core::*;
use nrg_platform::*;

fn main() {
    let mut app = App::new();

    {
        let path = PathBuf::from(library_filename("game_window"));
        app.add_plugin(path);
    }

    {
        let path = PathBuf::from(library_filename("game_renderer"));
        app.add_plugin(path);
    }

    {
        let path = PathBuf::from(library_filename("nrg_editor"));
        app.add_plugin(path);
    }

    {
        let path = PathBuf::from(library_filename("nrg_game"));
        app.add_plugin(path);
    }

    app.run();
}
