#![cfg(target_os = "windows")]

use std::{path::PathBuf, sync::Arc, thread};

use inox_binarizer::Binarizer;
use inox_commands::CommandParser;
use inox_filesystem::library_filename;
use inox_messenger::MessageHubRc;
use inox_profiler::debug_log;
use inox_resources::SharedDataRc;

use crate::launcher::Launcher;

pub fn setup_env() {
    std::env::set_var(
        inox_filesystem::EXE_PATH,
        std::env::current_exe().unwrap().parent().unwrap(),
    );
    std::env::set_current_dir(".").ok();
}

pub fn binarizer_start(shared_data: SharedDataRc, message_hub: MessageHubRc) -> Binarizer {
    debug_log!("Binarizing");
    let mut binarizer = Binarizer::new(
        &shared_data,
        &message_hub,
        inox_resources::Data::data_raw_folder(),
        inox_resources::Data::data_folder(),
    );
    binarizer.start();
    binarizer
}

pub fn binarizer_update(binarizer: Binarizer) -> Binarizer {
    while !binarizer.is_running() {
        thread::yield_now();
    }
    binarizer
}

pub fn binarizer_stop(mut binarizer: Binarizer) {
    binarizer.stop();
}

pub fn load_plugins(launcher: &Arc<Launcher>) {
    debug_log!("Loading plugins");

    //additional plugins
    let command_parser = CommandParser::from_command_line();
    let plugins = command_parser.get_values_of::<String>("plugin");

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(name));
        launcher.add_dynamic_plugin(name, path.as_path());
    }
}

pub fn main_update(launcher: Arc<Launcher>) {
    loop {
        let can_continue = launcher.update();
        if !can_continue {
            break;
        }
    }
}
