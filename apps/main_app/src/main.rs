use std::path::PathBuf;

use nrg_core::*;
use nrg_platform::*;

fn main() {
    let mut app = App::new();

    let plugins = [
        "nrg_core",
        "nrg_commands",
        "nrg_graphics",
        "nrg_gui",
        "nrg_math",
        "nrg_platform",
        "nrg_serialize",
        "nrg_window",
        "nrg_renderer",
        "nrg_profiler",
    ];

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

    app.run();
}
