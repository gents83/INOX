use std::path::PathBuf;

use nrg_binarizer::Binarizer;
use nrg_core::*;
use nrg_dynamic_library::library_filename;
use nrg_editor::editor::Editor;
use nrg_resources::{DATA_FOLDER, DATA_RAW_FOLDER};

fn main() {
    let mut app = App::new();

    let mut binarizer = Binarizer::new(app.get_global_messenger(), DATA_RAW_FOLDER, DATA_FOLDER);
    binarizer.start();

    let plugins = ["nrg_window"];

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

    let mut editor = Editor::default();
    editor.prepare(&mut app);

    loop {
        let can_continue = app.run_once();

        if !can_continue {
            break;
        }
    }

    editor.unprepare(&mut app);

    binarizer.stop();
}
