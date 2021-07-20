use std::path::PathBuf;

use nrg_core::*;
use nrg_dynamic_library::library_filename;
use nrg_test::entry_point::EntryPoint;

fn main() {
    let mut app = App::new();

    let plugins = ["nrg_window"];

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

    let mut entry_point = EntryPoint::default();
    entry_point.prepare(&mut app);

    loop {
        let can_continue = app.run_once();

        if !can_continue {
            break;
        }
    }

    entry_point.unprepare(&mut app);
}
