use std::path::PathBuf;

use nrg_binarizer::Binarizer;
use nrg_core::App;
use nrg_filesystem::library_filename;
use nrg_launcher::launcher::Launcher;
use nrg_resources::{DATA_FOLDER, DATA_RAW_FOLDER};

fn main() {
    let mut app = App::new();
    let mut plugins: Vec<&str> = Vec::new();

    let mut binarizer = Binarizer::new(app.get_global_messenger(), DATA_RAW_FOLDER, DATA_FOLDER);
    binarizer.start();

    let mut launcher = Launcher::default();
    launcher.prepare(&mut app);

    //additional plugins
    plugins.push("nrg_editor");

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

    launcher.unprepare(&mut app);

    binarizer.stop();
}
