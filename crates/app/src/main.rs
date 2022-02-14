use std::path::PathBuf;

use inox_commands::CommandParser;
use inox_core::App;
use inox_filesystem::library_filename;
use inox_launcher::{launcher::Launcher, platform::*};

fn main() {
    setup_env();
    
    let mut app = App::default();

    let mut launcher = Launcher::default();
    launcher.prepare(&mut app);

    //additional plugins
    let command_parser = CommandParser::from_command_line();
    let plugins = command_parser.get_values_of::<String>("plugin");

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(name));
        launcher.read_config(&mut app, name);
        app.add_plugin(path);
    }

    let mut binarizer = binarizer_start(&app);
    binarizer = binarizer_update(binarizer);

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
