use std::{env, path::PathBuf, thread};

use inox_binarizer::Binarizer;
use inox_commands::CommandParser;
use inox_core::App;
use inox_filesystem::{library_filename, EXE_PATH};
use inox_launcher::launcher::Launcher;
use inox_resources::Data;

fn main() {
    env::set_var(EXE_PATH, env::current_exe().unwrap().parent().unwrap());
    env::set_current_dir(".").ok();

    let mut app = App::default();

    let mut binarizer = Binarizer::new(
        app.get_message_hub(),
        Data::data_raw_folder(),
        Data::data_folder(),
    );
    binarizer.start();

    //additional plugins
    let command_parser = CommandParser::from_command_line();
    let plugins = command_parser.get_values_of::<String>("plugin");

    let mut launcher = Launcher::default();
    launcher.prepare(&mut app);

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(name));
        launcher.read_config(&mut app, name);
        app.add_plugin(path);
    }

    app.start();

    while !binarizer.is_running() {
        thread::yield_now();
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
