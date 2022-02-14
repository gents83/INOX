#![cfg(target_os = "windows")]

use std::thread;

use inox_binarizer::Binarizer;
use inox_core::App;

pub fn setup_env() {
    std::env::set_var(
        inox_filesystem::EXE_PATH,
        std::env::current_exe().unwrap().parent().unwrap(),
    );
    std::env::set_current_dir(".").ok();
}

pub fn binarizer_start(app: &App) -> Binarizer {
    let mut binarizer = Binarizer::new(
        app.get_shared_data(),
        app.get_message_hub(),
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
