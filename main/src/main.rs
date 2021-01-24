use std::path::PathBuf;

use nrg_app::*;
use nrg_platform::*;

fn main() {
    let mut app = App::new();

    let _game_plugin = load_plugin(&mut app, "C:\\PROJECTS\\NRG\\game\\target\\debug\\nrg_game.dll");
    
    app.run_once();

    let file_watcher = FileWatcher::new(PathBuf::from("C:\\PROJECTS\\NRG\\data\\"));

    for event in file_watcher.read_events() {
        match event {
            FileEvent::RenamedFrom(path) => println!("Renaming from {}", path.to_str().unwrap()),
            FileEvent::RenamedTo(path) => println!("Renamed into {}", path.to_str().unwrap()),
            FileEvent::Created(path) => println!("Created {}", path.to_str().unwrap()),
            FileEvent::Deleted(path) => println!("Deleted {}", path.to_str().unwrap()),
            FileEvent::Modified(path) => println!("Modified {}", path.to_str().unwrap()),
            _ => println!("Undefined event"),
        }
    }

}