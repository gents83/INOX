use std::path::PathBuf;

use nrg_core::*;
use nrg_platform::*;

fn main() {
    let mut app = App::new();

    let plugins = [
        "nrg_core",
        "nrg_graphics",
        "nrg_gui",
        "nrg_math",
        "nrg_platform",
        "nrg_serialize",
        "game_window",
        "game_renderer",
        "nrg_editor",
        "nrg_game",
    ];

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

    app.run();
}
