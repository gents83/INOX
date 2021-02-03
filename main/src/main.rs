use std::path::PathBuf;

use nrg_app::*;
use nrg_platform::*;


fn main() {
    let mut app = App::new();

    {
        let path = PathBuf::from(library_filename("nrg_window"));
        app.add_plugin(path);
    }

    {
        let path = PathBuf::from(library_filename("nrg_game"));
        app.add_plugin(path);
    }

    {
        let path = PathBuf::from(library_filename("nrg_graphics"));
        app.add_plugin(path);
    }
    
    app.run();
}