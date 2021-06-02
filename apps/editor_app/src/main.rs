use std::path::PathBuf;

use nrg_binarizer::Binarizer;
use nrg_core::*;
use nrg_dynamic_library::library_filename;
use nrg_resources::{DATA_FOLDER, DATA_RAW_FOLDER};

fn main() {
    let mut app = App::new();

    let mut binarizer = Binarizer::new(app.get_global_messenger(), DATA_RAW_FOLDER, DATA_FOLDER);
    binarizer.start();

    let plugins = [
        "nrg_binarizer",
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
        "nrg_editor",
    ];

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

    loop {
        let can_continue = app.run_once();

        if !can_continue {
            break;
        }
    }

    binarizer.stop();
}
