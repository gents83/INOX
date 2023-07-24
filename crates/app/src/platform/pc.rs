#![cfg(target_os = "windows")]

use std::{path::PathBuf, sync::Arc};

use inox_commands::CommandParser;
use inox_filesystem::library_filename;
use inox_log::debug_log;

use crate::launcher::Launcher;

pub fn setup_env() {
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_BACKTRACE", "1");

    std::env::set_var(
        inox_filesystem::EXE_PATH,
        std::env::current_exe().unwrap().parent().unwrap(),
    );
    std::env::set_current_dir(".").ok();
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

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main_update(launcher: Arc<Launcher>) {
    #[cfg(feature = "dhat-heap")]
    println!("DHAT profiler enabled");
    
    #[cfg(feature = "dhat-heap")]
    let profiler = dhat::Profiler::new_heap();

    loop {
        let can_continue = launcher.update();
        if !can_continue {
            break;
        }
    }

    #[cfg(feature = "dhat-heap")]
    drop(profiler);
}
