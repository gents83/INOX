use std::path::PathBuf;

use inox_commands::CommandParser;
use inox_core::App;
use inox_filesystem::library_filename;
use inox_launcher::{launcher::Launcher, platform::*};
use inox_profiler::debug_log;

fn main() {
    setup_env();

    let mut app = App::default();

    debug_log!("Creating launcher");

    let mut launcher = Launcher::default();
    launcher.prepare(&mut app);

    debug_log!("Prepared launcher");

    //additional plugins
    let command_parser = CommandParser::from_command_line();
    let plugins = command_parser.get_values_of::<String>("plugin");

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(name));
        launcher.read_config(&mut app, name);
        app.add_plugin(path);
    }

    debug_log!("Binarizing");

    let mut binarizer = binarizer_start(&app);
    binarizer = binarizer_update(binarizer);

    debug_log!("Starting app");

    app.start();

    loop {
        let can_continue = app.run_once();

        if !can_continue {
            break;
        }
    }

    launcher.unprepare(&mut app);

    binarizer_stop(binarizer);
}
