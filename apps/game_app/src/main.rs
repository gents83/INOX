use std::path::PathBuf;

use nrg_core::*;
use nrg_dynamic_library::library_filename;

fn main() {
    let mut app = App::new();

    let plugins = [
        "nrg_core",
        "nrg_events",
        "nrg_graphics",
        "nrg_gui",
        "nrg_math",
        "nrg_messenger",
        "nrg_platform",
        "nrg_profiler",
        "nrg_resources",
        "nrg_serialize",
        "nrg_window",
        "nrg_game",
    ];

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

    app.run();
}
