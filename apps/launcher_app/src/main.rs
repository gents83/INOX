use std::{env, path::PathBuf, thread};

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
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut is_plugin = false;
        (1..args.len()).for_each(|i| {
            if args[i].starts_with("-plugin") {
                is_plugin = true;
                if let Some(plugin_name) = args[i].strip_prefix("-plugin ") {
                    plugins.push(plugin_name);
                    is_plugin = false;
                }
            } else if is_plugin {
                plugins.push(args[i].as_str());
                is_plugin = false;
            }
        });
    }

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

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
