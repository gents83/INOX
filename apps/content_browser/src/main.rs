use std::path::PathBuf;

use nrg_content_browser::entry_point::EntryPoint;
use nrg_core::*;
use nrg_filesystem::library_filename;

fn main() {
    let mut app = App::new();

    let plugins = ["nrg_window"];

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

    let mut content_browser = EntryPoint::default();
    content_browser.prepare(&mut app);

    loop {
        let can_continue = app.run_once();

        if !can_continue {
            break;
        }
    }

    content_browser.unprepare(&mut app);
}
