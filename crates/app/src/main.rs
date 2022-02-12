use std::{path::PathBuf, thread};

use inox_binarizer::Binarizer;
use inox_commands::CommandParser;
use inox_core::App;
use inox_filesystem::library_filename;
use inox_launcher::launcher::Launcher;

fn run() {
    let mut _binarizer: Option<Binarizer> = None;
    #[cfg(all(not(target_arch = "wasm32")))]
    {
        std::env::set_var(
            inox_filesystem::EXE_PATH,
            std::env::current_exe().unwrap().parent().unwrap(),
        );
        std::env::set_current_dir(".").ok();
    }

    let mut app = App::default();

    #[cfg(all(not(target_arch = "wasm32")))]
    {
        _binarizer = {
            let mut binarizer = Binarizer::new(
                app.get_shared_data(),
                app.get_message_hub(),
                inox_resources::Data::data_raw_folder(),
                inox_resources::Data::data_folder(),
            );
            binarizer.start();
            Some(binarizer)
        };
    }

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

    if let Some(binarizer) = &mut _binarizer {
        while !binarizer.is_running() {
            thread::yield_now();
        }
    }

    loop {
        let can_continue = app.run_once();

        if !can_continue {
            break;
        }
    }

    launcher.unprepare(&mut app);

    if let Some(binarizer) = &mut _binarizer {
        binarizer.stop();
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        fn error(msg: String);

        type Error;

        #[wasm_bindgen(constructor)]
        fn new() -> Error;

        #[wasm_bindgen(structural, method, getter)]
        fn stack(error: &Error) -> String;
    }

    pub fn hook(info: &std::panic::PanicInfo) {
        hook_impl(info);
    }

    fn hook_impl(info: &std::panic::PanicInfo) {
        let mut msg = info.to_string();
        msg.push_str("\n\nStack:\n\n");
        let e = Error::new();
        let stack = e.stack();
        msg.push_str(&stack);
        msg.push_str("\n\n");
        error(msg);
    }

    #[inline]
    pub fn set_once() {
        use std::sync::Once;
        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            std::panic::set_hook(Box::new(hook));
        });
    }

    #[wasm_bindgen]
    pub fn init_panic_hook() {
        set_once();
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        wasm::init_panic_hook();
    }

    run();
}
