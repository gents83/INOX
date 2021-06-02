use std::path::PathBuf;

use nrg_binarizer::{ConfigCompiler, DataWatcher, FontCompiler, ImageCompiler, ShaderCompiler};
use nrg_core::*;
use nrg_dynamic_library::library_filename;
use nrg_resources::{DATA_FOLDER, DATA_RAW_FOLDER};

fn main() {
    let mut app = App::new();

    let mut binarizer = DataWatcher::new(DATA_RAW_FOLDER, DATA_FOLDER);

    let shader_compiler = ShaderCompiler::new(app.get_global_messenger());
    let config_compiler = ConfigCompiler::new(app.get_global_messenger());
    let font_compiler = FontCompiler::new(app.get_global_messenger());
    let image_compiler = ImageCompiler::new(app.get_global_messenger());
    binarizer.add_handler(config_compiler);
    binarizer.add_handler(shader_compiler);
    binarizer.add_handler(font_compiler);
    binarizer.add_handler(image_compiler);
    binarizer.binarize_all();

    let plugins = [
        "nrg_binarizer",
        "nrg_core",
        "nrg_events",
        "nrg_graphics",
        "nrg_gui",
        "nrg_math",
        "nrg_messenger",
        "nrg_platform",
        "nrg_profiler",
        "nrg_resources",
        "nrg_serialize",
        "nrg_window",
        "nrg_editor",
    ];

    for name in plugins.iter() {
        let path = PathBuf::from(library_filename(*name));
        app.add_plugin(path);
    }

    loop {
        let can_continue = app.run_once();
        binarizer.update();

        if !can_continue {
            break;
        }
    }
}
